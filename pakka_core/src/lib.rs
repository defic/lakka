

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
    let self_ty = &input.self_ty;

    // Extract the full generics, including where clause
    let full_generics = &input.generics;

    let ask_enum_name = format_ident!("{}AskMessage", name);
    let tell_enum_name = format_ident!("{}TellMessage", name);
    let actor_enum_name = format_ident!("{}Message", name);

    let handle_name = format_ident!("{}Handle", name);

    let mut ask_variants = quote! {};
    let mut tell_variants = quote! {};
    let mut ask_handlers = quote! {};
    let mut tell_handlers = quote! {};

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

            let async_code = if method.sig.asyncness.is_some() {
                quote! { .await }
            } else {
                quote! {}
            };

            match return_type {
                syn::ReturnType::Default => {
                    //Tell variants

                    tell_variants.extend(quote! {
                        #variant_name(#variant_fields),
                    });

                    tell_handlers.extend(quote! {
                        #tell_enum_name::#variant_name(#handler_args) => {
                            self.#method_name(#handler_args)#async_code;
                        },
                    });

                    handle_methods.extend(quote! {
                        pub async fn #method_name(&self #(, #cleaned_args)*) {
                            let _ = self.sender.send(#actor_enum_name::Tell(#tell_enum_name::#variant_name(#handle_args))).await;
                        }
                    });
                },
                syn::ReturnType::Type(_, ty) => {
                    //Ask variants

                    ask_variants.extend(quote! {
                        #variant_name(#variant_fields tokio::sync::oneshot::Sender<#ty>),
                    });

                    ask_handlers.extend(quote! {
                        #ask_enum_name::#variant_name(#handler_args resp) => {
                            let result = self.#method_name(#handler_args)#async_code;
                            let _ = resp.send(result);
                        },
                    });

                    handle_methods.extend(quote! {
                        pub async fn #method_name(&self #(, #cleaned_args)*) -> #ty {
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            let _ = self.sender.send(#actor_enum_name::Ask(#ask_enum_name::#variant_name(#handle_args tx))).await;
                            rx.await.expect("Actor task has been killed")
                        }
                    });
                }
            }
        }
    }

    let name_string = name.to_string();

    let expanded = quote! {

        #[derive(Clone, Debug)]
        pub struct #handle_name #impl_generics {
            sender: tokio::sync::mpsc::Sender<#actor_enum_name #ty_generics>,
        } #where_clause

        #[derive(Debug)]
        pub enum #ask_enum_name #impl_generics {
            #ask_variants
        } #where_clause

        // Tells are clonable for broadcasts
        #[derive(Debug, Clone)]
        pub enum #tell_enum_name #impl_generics {
            #tell_variants
        } #where_clause

        #[derive(Debug)]
        pub enum #actor_enum_name #impl_generics {
            Ask(#ask_enum_name #ty_generics),
            Tell(#tell_enum_name #ty_generics),
        } #where_clause

        impl #impl_generics #self_ty #where_clause {

            pub fn run(mut self) -> #handle_name #ty_generics {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<#actor_enum_name #ty_generics>(100);
                
                tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        self.handle_message(msg).await;
                    }
                    self.exit();
                });

                #handle_name { sender: tx }
            }

            // Broadcasts can only receive tells.
            pub fn run_with_broadcast_receiver(mut self, mut broadcast_rx: tokio::sync::broadcast::Receiver<#tell_enum_name #ty_generics>) -> #handle_name #ty_generics {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<#actor_enum_name #ty_generics>(100);
                
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            msg = rx.recv() => {
                                match msg {
                                    Some(msg) => self.handle_message(msg).await,
                                    None => {
                                        // The channel has closed, exit the loop
                                        break;
                                    }
                                }
                            },
                            result = broadcast_rx.recv() => {
                                match result {
                                    Ok(msg) => self.handle_tells(msg).await,
                                    Err(err) => match err {
                                        tokio::sync::broadcast::error::RecvError::Closed => {
                                            break;
                                        },
                                        tokio::sync::broadcast::error::RecvError::Lagged(skipped_messages) => {
                                            // The broadcast channel lagging
                                            eprintln!("{} broadcast receiver lagging, skipped: {} messages", #name_string, skipped_messages);
                                        },
                                    },
                                }
                            }
                        } 
                    }

                    self.exit();
                });

                #handle_name { sender: tx }
            }

            fn exit(&self) {
                println!("{} actor task exiting", #name_string);
            }

            async fn handle_message(&mut self, msg: #actor_enum_name #ty_generics) {
                match msg {
                    #actor_enum_name::Ask(ask_msg) => self.handle_asks(ask_msg).await,
                    #actor_enum_name::Tell(tell_msg) => self.handle_tells(tell_msg).await,
                }
            }

            async fn handle_asks(&mut self, msg: #ask_enum_name #ty_generics) {
                match msg {
                    #ask_handlers
                }
            }

            async fn handle_tells(&mut self, msg: #tell_enum_name #ty_generics) {
                match msg {
                    #tell_handlers
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
