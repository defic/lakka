use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn messages(args: TokenStream, input: TokenStream) -> TokenStream {
    pakka_core::messages(args.into(), input.into()).into()
}
