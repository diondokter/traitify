#![doc = include_str!(env!("CARGO_PKG_README"))]

#[proc_macro_attribute]
pub fn traitify(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    traitify_core::traitify(args.into(), input.into()).into()
}
