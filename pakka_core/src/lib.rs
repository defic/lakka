use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, FnArg, ImplItem, ItemImpl, ItemStruct, Pat, Receiver, Type, WhereClause, WherePredicate};
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

fn to_snake_case(s: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

pub fn messages(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let mut input = match syn::parse2::<ItemImpl>(item) {
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

    let (name, full_type) = if let Type::Path(type_path) = &*input.self_ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            (&last_segment.ident, &input.self_ty)
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
    let has_generics = !generics.params.is_empty();
    let self_ty = &input.self_ty;

    let type_string = quote! { #full_type }.to_string().replace([' ', '<', '>'], "").replace("::", "");
    let ask_enum_name = format_ident!("{}AskMessage", name);
    let tell_enum_name = format_ident!("{}TellMessage", name);
    //let actor_enum_name = format_ident!("{}Message", name);
    let handle_name = format_ident!("{}Handle", name);

    let module_name = format_ident!("{}", to_snake_case(&type_string));

    let mut ask_variants = quote! {};
    let mut tell_variants = quote! {};
    let mut ask_handlers = quote! {};
    let mut tell_handlers = quote! {};

    let mut handle_methods = quote! {};

    for item in &mut input.items {
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
                            self.#method_name(#handler_args &mut _ctx)#async_code;
                        },
                    });

                    handle_methods.extend(quote! {
                        pub async fn #method_name(&self #(, #cleaned_args)*) -> Result<(), pakka::ActorError> {
                            self.sender.send(pakka::Message::Tell(#tell_enum_name::#variant_name(#handle_args))).await?;
                            Ok(())
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
                            let result = self.#method_name(#handler_args &mut _ctx)#async_code;
                            let _ = resp.send(result);
                        },
                    });

                    handle_methods.extend(quote! {
                        pub async fn #method_name(&self #(, #cleaned_args)*) -> Result<(#ty), pakka::ActorError> {
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            self.sender.send(pakka::Message::Ask(#ask_enum_name::#variant_name(#handle_args tx))).await?;
                            rx.await.map_err(Into::into)
                        }
                    });
                }
            }

            //let method_clone = method.clone();
            //let new_param: syn::FnArg = syn::parse_quote!(_ctx: &mut pakka::ActorCtx<impl pakka::Channel<#module_name::#actor_enum_name #ty_generics>, #module_name::#actor_enum_name #ty_generics>);
            let new_param: syn::FnArg = syn::parse_quote!(_ctx: &mut pakka::ActorContext<Self>);
            method.sig.inputs.push(new_param);
        }
    }

    let handle_ask_enum_phantom = has_generics.then(|| {
        quote! {
            #ask_enum_name::__Phantom(_) => (),
        }
    });

    let handle_tell_enum_phantom = has_generics.then(|| {
        quote! {
            #tell_enum_name::__Phantom(_) => (),
        }
    });

    let enum_phantom_field = has_generics.then(|| {
        quote! {
            #[doc(hidden)]
            __Phantom(PhantomData #ty_generics),
        }
    });

    let name_string = name.to_string();

    let expanded = quote! {
        mod #module_name {
            use std::marker::PhantomData;

            use super::*;

            impl #impl_generics pakka::Actor for #self_ty {
                type Ask = #ask_enum_name #ty_generics;
                type Tell = #tell_enum_name #ty_generics;
                type Handle = #handle_name #ty_generics;

                async fn handle_asks(&mut self, msg: Self::Ask, mut _ctx: &mut pakka::ActorContext<Self>) {
                    match msg {
                        #ask_handlers
                        #handle_ask_enum_phantom
                    }
                }

                async fn handle_tells(&mut self, msg: Self::Tell, mut _ctx: &mut pakka::ActorContext<Self>) {
                    match msg {
                        #tell_handlers
                        #handle_tell_enum_phantom
                    }
                }
            }

            #[derive(Clone, Debug)]
            pub struct #handle_name #impl_generics #where_clause {
                sender: Box<dyn pakka::ChannelSender<Message #ty_generics>>,
            }

            //impl #impl_generics #handle_name #ty_generics #where_clause {

            impl #impl_generics pakka::ActorHandle<Message #ty_generics> for #handle_name #ty_generics #where_clause {
                fn new(tx: Box<dyn pakka::ChannelSender<Message #ty_generics>>) -> Self {
                    Self {
                        sender: tx
                    }
                }
            }


            #[derive(Debug)]
            pub enum #ask_enum_name #impl_generics {
                #ask_variants
                #enum_phantom_field
            } #where_clause

            // Tells are clonable for broadcasts
            #[derive(Debug, Clone)]
            pub enum #tell_enum_name #impl_generics {
                #tell_variants
                #enum_phantom_field
            } #where_clause

            type Message #ty_generics = pakka::Message<<#self_ty as pakka::Actor>::Ask, <#self_ty as pakka::Actor>::Tell>;

            impl #impl_generics #handle_name #ty_generics #where_clause {
                
                #handle_methods
            }
        }

        pub use #module_name::*;

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
