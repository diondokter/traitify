//! See the [readme of the git repo](https://github.com/diondokter/traitify) for information about the crate

/// ```rust
/// use traitify::traitify;
/// 
/// struct Foo<BAR, const FIZZ: usize> {
///     data: [BAR; FIZZ],
/// };
/// 
/// #[traitify(FooDynFizz, dyn = [FIZZ])]
/// #[traitify(FooDynBAR, dyn = [BAR])]
/// #[traitify(FooDynAll, dyn = [FIZZ, BAR])]
/// impl<BAR, const FIZZ: usize> Foo<BAR, FIZZ> {
///     pub fn print_greeting(&self) {
///         println!("Hallo vriendjes en vriendinnetjes!");
///     }
/// }
/// 
/// #[test]
/// fn compiles() {
///     let foo = Foo::<u8, 4> { data: [0; 4] };
///     let dyn_foo: &dyn FooDynAll = &foo;
///     dyn_foo.print_greeting();
/// }
/// ```
#[proc_macro_attribute]
pub fn traitify(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    traitify_core::traitify(args.into(), input.into()).into()
}
