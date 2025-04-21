use proc_macro2::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{GenericArgument, ItemImpl, Meta, Path, PathArguments, Type};

use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_quote, FnArg, GenericParam, ImplItem, ItemStruct, Pat, Receiver, WhereClause,
    WherePredicate,
};

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

fn extract_generic_types(item_impl: &ItemImpl) -> TokenStream {
    if let Type::Path(type_path) = &*item_impl.self_ty {
        if let Some(segment) = type_path.path.segments.last() {
            if let PathArguments::AngleBracketed(generic_args) = &segment.arguments {
                let types: Vec<_> = generic_args.args.iter().collect();
                if !types.is_empty() {
                    return quote!(<#(#types),*>);
                }
            }
        }
    }
    quote!()
}

fn extract_generic_params(item_impl: &ItemImpl) -> Vec<GenericArgument> {
    if let Type::Path(type_path) = &*item_impl.self_ty {
        if let Some(segment) = type_path.path.segments.last() {
            if let PathArguments::AngleBracketed(generic_args) = &segment.arguments {
                return generic_args.args.iter().cloned().collect();
            }
        }
    }
    vec![]
}

pub fn messages(attr: TokenStream, item: TokenStream) -> TokenStream {
    let meta = syn::parse2::<Meta>(attr).unwrap_or_else(|_| {
        Meta::Path(Path::from(syn::Ident::new(
            "default",
            proc_macro2::Span::call_site(),
        )))
    });

    let mut unbounded = false;
    match meta {
        Meta::Path(path) if path.is_ident("unbounded") => {
            unbounded = true;
        }
        Meta::Path(path) if path.is_ident("default") => {}
        _ => {
            let error = syn::Error::new(
                proc_macro2::Span::call_site(),
                "Invalid attribute parameter. Use #[lakka::messages(unbounded)] or #[lakka::messages] only".to_string(),
            );
            return error.to_compile_error();
        }
    }

    let mut input = match syn::parse2::<ItemImpl>(item) {
        Ok(ast) => ast,
        Err(e) => {
            let error = syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Error with parsing input: {e}"),
            );
            return error.to_compile_error();
        }
    };

    let (name, full_type) = if let Type::Path(type_path) = &*input.self_ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            (&last_segment.ident, &input.self_ty)
        } else {
            return syn::Error::new(Span::call_site(), "Unable to determine actor name")
                .to_compile_error();
        }
    } else {
        return syn::Error::new(Span::call_site(), "Unexpected type in impl block")
            .to_compile_error();
    };

    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let self_ty = &input.self_ty;

    let type_string = quote! { #full_type }
        .to_string()
        .replace([' ', '<', '>', ','], "")
        .replace("::", "");
    let ask_enum_name = format_ident!("{}AskMessage", name);
    let tell_enum_name = format_ident!("{}TellMessage", name);
    let message_name = format_ident!("{}Message", name);
    //let actor_enum_name = format_ident!("{}Message", name);

    let handle_name = format_ident!("{}Handle", name);
    let channel_sender_type = if unbounded {
        quote!(UnboundedChannelSender)
    } else {
        quote!(ChannelSender)
    };

    let mut actor_type = if unbounded {
        quote!(UnboundedActor)
    } else {
        quote!(BoundedActor)
    };

    let mut handle_type = if unbounded {
        quote!(UnboundedActorHandle)
    } else {
        quote!(ActorHandle)
    };
    //actor_type = quote!(Actor);

    //let unbounded_handle_name = format_ident!("{}HandleUnbounded", name);
    let generic_params = extract_generic_params(&input);
    let generic_types = extract_generic_types(&input);

    let type_always = if !ty_generics.to_token_stream().is_empty() {
        quote!( #ty_generics )
    } else {
        generic_types.clone()
    };

    let impl_generics_over_generic_types = if !impl_generics.to_token_stream().is_empty() {
        quote!( #impl_generics )
    } else {
        generic_types.clone()
    };

    //let prefer_ty_generics =

    //let handle_name = quote! { #handle_name #generic_types };

    // Generate PhantomData fields for generic parameters for ActorHandle
    let phantom_types: Vec<_> = generic_params
        .iter()
        .map(|param| {
            match param {
                GenericArgument::Type(ty) => quote! { std::marker::PhantomData<#ty> },
                GenericArgument::Lifetime(lifetime) => {
                    quote! { std::marker::PhantomData<&#lifetime ()> }
                }
                _ => quote! {}, // Ignore other cases
            }
        })
        .collect();

    // Generate PhantomData field initializers for the constructor
    let (phantom_field, phantom_init, enum_phantom_field) = if phantom_types.is_empty() {
        (quote! {}, quote! {}, quote! {})
    } else {
        (
            quote! { _phantom: (#(#phantom_types),*), },
            quote! { _phantom: Default::default(), },
            quote! { _Phantom((#(#phantom_types),*)) },
        )
    };

    let module_name = format_ident!("{}", to_snake_case(&type_string));

    let mut ask_variants = quote! {};
    let mut tell_variants = quote! {};
    let mut ask_handlers = quote! {};
    let mut tell_handlers = quote! {};

    let mut handle_methods = quote! {};
    //let mut unbounded_handle_methods = quote! {};

    for item in &mut input.items {
        if let ImplItem::Fn(method) = item {
            //Only &self and &mut self functions are turned into messages
            match method.sig.inputs.first() {
                Some(FnArg::Receiver(Receiver {
                    reference: Some(_),
                    colon_token: None,
                    ..
                })) => true,
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
            let unbounded_cleaned_args = cleaned_args.clone();

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

                    handle_methods.extend({
                        if !unbounded {
                            quote! {
                                pub async fn #method_name(&self #(, #cleaned_args)*) -> Result<(), lakka::ActorError> {
                                    self.sender.send(lakka::Message::Tell(#tell_enum_name::#variant_name(#handle_args))).await?;
                                    Ok(())
                                }
                            }
                        } else {
                            quote! {
                                pub fn #method_name(&self #(, #unbounded_cleaned_args)*) -> Result<(), lakka::ActorError> {
                                    self.sender.send(lakka::Message::Tell(#tell_enum_name::#variant_name(#handle_args)))?;
                                    Ok(())
                                }
                            }
                        }
                    });
                }
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

                    handle_methods.extend({
                        if !unbounded {
                            quote! {
                                pub async fn #method_name(&self #(, #cleaned_args)*) -> Result<(#ty), lakka::ActorError> {
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    self.sender.send(lakka::Message::Ask(#ask_enum_name::#variant_name(#handle_args tx))).await?;
                                    rx.await.map_err(Into::into)
                                }
                            }
                        } else {
                            quote! {
                                pub async fn #method_name(&self #(, #unbounded_cleaned_args)*) -> Result<(#ty), lakka::ActorError> {
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    self.sender.send(lakka::Message::Ask(#ask_enum_name::#variant_name(#handle_args tx)))?;
                                    rx.await.map_err(Into::into)
                                }
                            }
                        }
                    });
                }
            }

            //let method_clone = method.clone();
            //let new_param: syn::FnArg = syn::parse_quote!(_ctx: &mut lakka::ActorCtx<impl lakka::Channel<#module_name::#actor_enum_name #ty_generics>, #module_name::#actor_enum_name #ty_generics>);
            let new_param: syn::FnArg =
                syn::parse_quote!(_ctx: &mut lakka::ActorContext<Self>);
            method.sig.inputs.push(new_param);
        }
    }

    let handle_ask_enum_phantom = (!phantom_types.is_empty()).then(|| {
        quote! {
            #ask_enum_name::_Phantom(_) => (),
        }
    });

    let handle_tell_enum_phantom = (!phantom_types.is_empty()).then(|| {
        quote! {
            #tell_enum_name::_Phantom(_) => (),
        }
    });

    /*
    let enum_phantom_field = (!ty_generics.to_token_stream().is_empty()).then(|| {
        quote! {
            #[doc(hidden)]
            __Phantom(PhantomData #ty_generics),
        }
    });
     */

    let name_string = name.to_string();

    let expanded = quote! {
        mod #module_name {
            use super::*;

            impl #impl_generics lakka::#actor_type for #self_ty {
                type Handle = #handle_name #type_always;
            }

            impl #impl_generics lakka::Actor for #self_ty {
                type Ask = #ask_enum_name #type_always;
                type Tell = #tell_enum_name #type_always;
                //type Handle = #handle_name #type_always;

                async fn handle_asks(&mut self, msg: Self::Ask, mut _ctx: &mut lakka::ActorContext<Self>) {
                    match msg {
                        #ask_handlers
                        #handle_ask_enum_phantom
                    }
                }

                async fn handle_tells(&mut self, msg: Self::Tell, mut _ctx: &mut lakka::ActorContext<Self>) {
                    match msg {
                        #tell_handlers
                        #handle_tell_enum_phantom
                    }
                }
            }

            #[derive(Clone, Debug)]
            pub struct #handle_name #impl_generics_over_generic_types #where_clause {
                sender: Box<dyn lakka::#channel_sender_type<#message_name #ty_generics>>,
                #phantom_field
            }
            impl #impl_generics #handle_name #type_always #where_clause {
                #handle_methods
            }
            impl #impl_generics lakka::#handle_type<#message_name #ty_generics> for #handle_name #type_always #where_clause {
                fn new(tx: Box<dyn lakka::#channel_sender_type<#message_name #ty_generics>>) -> Self {
                    Self {
                        sender: tx,
                        #phantom_init
                    }
                }
            }

            //TODO: cfgfeature
            /*
            #[derive(Clone, Debug)]
            pub struct #unbounded_handle_name #impl_generics_over_generic_types #where_clause {
                sender: Box<dyn lakka::UnboundedChannelSender<#message_name #ty_generics>>,
                #phantom_field
            }
            impl #impl_generics #unbounded_handle_name #type_always #where_clause {
                #unbounded_handle_methods
            }
            impl #impl_generics lakka::UnboundedActorHandle<#message_name #ty_generics> for #unbounded_handle_name #type_always #where_clause {
                fn new(tx: Box<dyn lakka::UnboundedChannelSender<#message_name #ty_generics>>) -> Self {
                    Self {
                        sender: tx,
                        #phantom_init
                    }
                }
            }
            */


            #[derive(Debug)]
            pub enum #ask_enum_name #impl_generics_over_generic_types {
                #ask_variants
                #enum_phantom_field
            } #where_clause

            // Tells are clonable for broadcasts
            #[derive(Debug, Clone)]
            pub enum #tell_enum_name #impl_generics_over_generic_types {
                #tell_variants
                #enum_phantom_field
            } #where_clause

            type #message_name #ty_generics = lakka::Message<<#self_ty as lakka::Actor>::Ask, <#self_ty as lakka::Actor>::Tell>;
        }

        pub use #module_name::*;

        #[allow(dead_code)]
        #input
    };

    expanded
}

fn args_to_fields_and_args(
    args: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
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
