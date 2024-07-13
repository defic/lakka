

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{FnArg, ImplItem, ItemImpl, ItemStruct, Pat, Receiver, Type};
use proc_macro2::Span;

fn to_pascal_case(s: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            pascal.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            pascal.push(c);
        }
    }
    pascal
}

pub fn actor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = match syn::parse2::<ItemStruct>(item) {
        Ok(ast) => ast,
        Err(e) => {
            let error = syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Error with parsing macro: {e}")
            );
            eprintln!("Error: {}", error);
            return error.to_compile_error();
        }
    };

    /* new field 
    let new_field: Field = syn::parse_quote! {
        pub new_member: String
    };

    
    if let syn::Fields::Named(ref mut fields) = input.fields {
        fields.named.push(new_field);
    } else {
        let error = syn::Error::new(
            proc_macro2::Span::call_site(),
            "Only named structs can be actors"
        );
        eprintln!("Error: {}", error);
        return error.to_compile_error();
    }
    */

    //let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (_, ty_generics, _) = generics.split_for_impl();
    let full_generics = &input.generics;
    //input.fields

    let message_enum_name = format_ident!("{}Message", name);
    let handle_name = format_ident!("{}Handle", name);


    /* 
    // Ensure we're dealing with a struct
    match input.data {
        Data::Struct(_) => {},
        _ => {
            let error = syn::Error::new(
                proc_macro2::Span::call_site(),
                "The actor attribute can only be applied to structs"
            );
            eprintln!("Error: {}", error);
            return error.to_compile_error();
        }
    }
    */

    let expanded = quote! {
        #input

        /* TODO: Implement actor trait?
        impl #full_generics Actor for #name {
        
        }
        */


        #[derive(Clone)]
        pub struct #handle_name #full_generics {
            sender: tokio::sync::mpsc::Sender<#message_enum_name #ty_generics>,
        }
    };

    expanded
}

pub fn messages(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let input = match syn::parse2::<ItemImpl>(item) {
        Ok(ast) => ast,
        Err(e) => {
            let error = syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Error with parsing input: {e}")
            );
            eprintln!("Error: {}", error);
            return error.to_compile_error();
        }
    };

    let name = if let Type::Path(type_path) = &*input.self_ty {
        if let Some(segment) = type_path.path.segments.last() {
            &segment.ident
        } else {
            return syn::Error::new(
                Span::call_site(),
                "Unable to determine actor name"
            ).to_compile_error();
        }
    } else {
        return syn::Error::new(
            Span::call_site(),
            "Unexpected type in impl block"
        ).to_compile_error();
    };


    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract the full generics, including where clause
    let full_generics = &input.generics;

    let message_enum_name = format_ident!("{}Message", name);
    let handle_name = format_ident!("{}Handle", name);

    let mut message_variants = quote! {};
    let mut message_handlers = quote! {};
    let mut handle_methods = quote! {};

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            
            //Only &self and &mut self functions are turned into messages
            match method.sig.inputs.first() {
                Some(FnArg::Receiver(Receiver { reference: Some(_), colon_token: None, .. })) => true,
                _ => continue,
            };
            
            let method_name = &method.sig.ident;
            let args = &method.sig.inputs;
            let return_type = &method.sig.output;

            //Remove first element of the arguments (&self / &mut self)
            let mut iterator = args.iter();
            _ = iterator.next();
            let remaining_args: Vec<_> = iterator.collect();
            let cleaned_args = remaining_args.iter().map(|arg| {
                quote! { #arg }
            });

            let variant_name = format_ident!("{}", to_pascal_case(&method_name.to_string()));
            
            let (variant_fields, handler_args, handle_args) = args_to_fields_and_args(args);

            match return_type {
                syn::ReturnType::Default => {
                    message_variants.extend(quote! {
                        #variant_name(#variant_fields),
                    });

                    message_handlers.extend(quote! {
                        #message_enum_name::#variant_name(#handler_args) => {
                            self.#method_name(#handler_args);
                        },
                    });

                    handle_methods.extend(quote! {
                        pub async fn #method_name(&self #(, #cleaned_args)*) {
                            let _ = self.sender.send(#message_enum_name::#variant_name(#handle_args)).await;
                        }
                    });
                },
                syn::ReturnType::Type(_, ty) => {
                    message_variants.extend(quote! {
                        #variant_name(#variant_fields tokio::sync::oneshot::Sender<#ty>),
                    });

                    message_handlers.extend(quote! {
                        #message_enum_name::#variant_name(#handler_args resp) => {
                            let result = self.#method_name(#handler_args);
                            let _ = resp.send(result);
                        },
                    });

                    handle_methods.extend(quote! {
                        pub async fn #method_name(&self #(, #cleaned_args)*) -> #ty {
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            let _ = self.sender.send(#message_enum_name::#variant_name(#handle_args tx)).await;
                            rx.await.expect("Actor task has been killed")
                        }
                    });
                }
            }
        }
    }

    let expanded = quote! {
        #[derive(Debug)]
        pub enum #message_enum_name #full_generics {
            #message_variants
        }

        impl #impl_generics #name #ty_generics #where_clause {

            pub fn run(mut self) -> #handle_name #ty_generics {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<#message_enum_name #ty_generics>(100);
                
                tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        self.handle_message(msg);
                    }
                });

                #handle_name { sender: tx }
            }

            pub fn handle_message(&mut self, msg: #message_enum_name #ty_generics) {
                match msg {
                    #message_handlers
                }
            }
        }


        impl #impl_generics #handle_name #ty_generics #where_clause {
            #handle_methods
        }

        #[allow(dead_code)]
        #input
    };

    expanded
}

fn args_to_fields_and_args(args: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) 
    -> (proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut fields = quote! {};
    let mut handler_args = quote! {};
    let mut handle_args = quote! {};

    for arg in args {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let ident = &pat_ident.ident;
                let ty = &pat_type.ty;
                // Skip 'self' parameter for handle methods
                if ident != "self" {
                    fields.extend(quote! { #ty, });
                    handler_args.extend(quote! { #ident, });
                    handle_args.extend(quote! { #ident, });
                }
            }
        }
    }

    (fields, handler_args, handle_args)
}
