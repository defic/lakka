use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
    pakka_core::actor(args.into(), input.into()).into()
}

#[proc_macro_attribute]
pub fn messages(args: TokenStream, input: TokenStream) -> TokenStream {
    pakka_core::messages(args.into(), input.into()).into()
}
