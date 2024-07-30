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
    let actor_enum_name = format_ident!("{}Message", name);
    let handle_name = format_ident!("{}Handle", name);

    // Modified handle generics to have pakka::ChannelSender
    let mut handle_generics = generics.clone();
    handle_generics.params.push(parse_quote!(S));
    
    //own generics for ActorHandle, as ChannelSender is generic
    let channel_sender_predicate: WherePredicate = parse_quote!(S: pakka::ChannelSender<#actor_enum_name #ty_generics>);
    let handle_where_clause = handle_generics.where_clause.get_or_insert_with(|| WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    handle_where_clause.predicates.push(channel_sender_predicate);
    let (handle_impl_generics, handle_ty_generics, mut handle_where_clause) = handle_generics.split_for_impl();


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
                            self.sender.send(#actor_enum_name::Tell(#tell_enum_name::#variant_name(#handle_args))).await?;
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
                            self.sender.send(#actor_enum_name::Ask(#ask_enum_name::#variant_name(#handle_args tx))).await?;
                            rx.await.map_err(Into::into)
                        }
                    });
                }
            }

            //let method_clone = method.clone();
            let new_param: syn::FnArg = syn::parse_quote!(_ctx: &mut pakka::ActorCtx<impl pakka::Channel<#module_name::#actor_enum_name #ty_generics>, #module_name::#actor_enum_name #ty_generics>);
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
            use pakka::channel::mpsc::{channel, Receiver, Sender};
            use std::marker::PhantomData;
            use futures::stream::*;
            use futures::FutureExt;

            use super::*;

            #[derive(Clone, Debug)]
            pub struct #handle_name #impl_generics #where_clause {
                sender: Sender<#actor_enum_name #ty_generics>,
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

            #[derive(Debug)]
            pub enum #actor_enum_name #impl_generics {
                Ask(#ask_enum_name #ty_generics),
                Tell(#tell_enum_name #ty_generics),
            } #where_clause

            //type ActorContext #impl_generics = pakka::ActorCtx<dyn pakka::Channel<#module_name::#actor_enum_name #ty_generics>, #actor_enum_name #ty_generics>;
            
            impl #impl_generics #self_ty #where_clause {

                pub fn run_with_channels(mut self, mut channels: Vec<Box<dyn pakka::Channel<#tell_enum_name #ty_generics>>>) -> #handle_name #ty_generics {
                    
                    let (tx, mut rx) = channel::<#actor_enum_name #ty_generics>(100);
                    
                    tokio::spawn(async move {
                        let mut ctx = pakka::ActorCtx::new(rx);

                        loop {
                            let mut remove_index = None;

                            //If we should poll multiple channels
                            if channels.len() > 0 {
                                let future = futures::future::select_all(
                                    channels
                                        .iter_mut()
                                        .map(|channel| channel.recv().boxed()),
                                );
    
                                tokio::select! {
                                    msg = ctx.rx.recv() => {
                                        //let mut ctx = pakka::ActorCtx::new(rx);
                                        match msg {
                                            Some(msg) => self.handle_message(msg, &mut ctx).await,
                                            None => {
                                                // The channel has closed, exit the loop
                                                break;
                                            }
                                        }
                                    },
                                    (result, index, _) = future => {
                                        match result {
                                            Ok(msg) => self.handle_tells(msg, &mut ctx).await,
                                            Err(error) => {
                                                println!("Channel died, removing index: {}", index);
                                                remove_index = Some(index);
                                            }
                                        }
                                    }
                                }
                                if let Some(index) = remove_index {
                                    println!("Channel died, removing index: {}, length is: {}", index, channels.len() );
                                    channels.remove(index);
                                    println!("Removed, now length {}", channels.len() );
                                }
                            }
                            else {
                                let msg = ctx.rx.recv().await;
                                match msg {
                                    Some(msg) => self.handle_message(msg, &mut ctx).await,
                                    None => {
                                        // The channel has closed, exit the loop
                                        break;
                                    }
                                }
                            }

                            if ctx.kill_flag {
                                break;
                            }
                        }

                        self.exit();
                    });

                    #handle_name { sender: tx }
                }

                pub fn run(mut self) -> #handle_name #ty_generics {
                    let (tx, mut rx) = channel::<#actor_enum_name #ty_generics>(100);
                    
                    tokio::spawn(async move {
                        let mut ctx = pakka::ActorCtx::new(rx);
                        while let Some(msg) = ctx.rx.recv().await {
                            self.handle_message(msg, &mut ctx).await;
                            if ctx.kill_flag {
                                break;
                            }
                        }
                        self.exit();
                    });

                    #handle_name { sender: tx }
                }

                pub fn run_with_broadcast_receiver(mut self, mut broadcast_rx: tokio::sync::broadcast::Receiver<#tell_enum_name #ty_generics>) -> #handle_name #ty_generics {
                    
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<#actor_enum_name #ty_generics>(100);   
                    tokio::spawn(async move {
                        let mut ctx = pakka::ActorCtx::new(rx);
                        loop {
                            tokio::select! {
                                msg = ctx.rx.recv() => {
                                    //let mut ctx = pakka::ActorCtx::new(rx);
                                    match msg {
                                        Some(msg) => self.handle_message(msg, &mut ctx).await,
                                        None => {
                                            // The channel has closed, exit the loop
                                            break;
                                        }
                                    }
                                },
                                result = broadcast_rx.recv() => {
                                    //let mut ctx = pakka::ActorCtx::new(rx);
                                    match result {
                                        Ok(msg) => self.handle_tells(msg, &mut ctx).await,
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
                            if ctx.kill_flag {
                                break;
                            }
                        }

                        self.exit();
                    });

                    #handle_name::new(tx)
                }
                

                fn exit(&self) {
                    println!("{} actor task exiting", #name_string);
                }

                async fn handle_message(&mut self, msg: #actor_enum_name #ty_generics, mut _ctx: &mut pakka::ActorCtx<impl pakka::Channel<#module_name::#actor_enum_name #ty_generics>, #actor_enum_name #ty_generics>) {
                    match msg {
                        #actor_enum_name::Ask(ask_msg) => self.handle_asks(ask_msg, &mut _ctx).await,
                        #actor_enum_name::Tell(tell_msg) => self.handle_tells(tell_msg, &mut _ctx).await,
                    }
                }

                async fn handle_asks(&mut self, msg: #ask_enum_name #ty_generics, mut _ctx: &mut pakka::ActorCtx<impl pakka::Channel<#module_name::#actor_enum_name #ty_generics>, #actor_enum_name #ty_generics>) {
                    match msg {
                        #ask_handlers
                        #handle_ask_enum_phantom
                    }
                }

                async fn handle_tells(&mut self, msg: #tell_enum_name #ty_generics, mut _ctx: &mut pakka::ActorCtx<impl pakka::Channel<#module_name::#actor_enum_name #ty_generics>, #actor_enum_name #ty_generics>) {
                    match msg {
                        #tell_handlers
                        #handle_tell_enum_phantom
                    }
                }
            }

            impl #impl_generics #handle_name #ty_generics #where_clause {
                pub fn new(sender: Sender<#actor_enum_name #ty_generics>) -> Self {
                    Self {
                        sender,
                    }
                }
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
