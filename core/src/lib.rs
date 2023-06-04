use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

#[derive(Debug)]
struct Args {
    trait_name: Ident,
    dyn_generics: Vec<String>,
}

impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_name = input.parse()?;
        input.parse::<syn::Token!(,)>()?;
        input.parse::<syn::Token!(dyn)>()?;
        input.parse::<syn::Token!(=)>()?;

        let generics_group;
        syn::bracketed!(generics_group in input);
        let dyn_generics = generics_group
            .parse_terminated(Ident::parse, syn::Token!(,))?
            .into_iter()
            .map(|ident| ident.to_string())
            .collect();

        Ok(Args {
            trait_name,
            dyn_generics,
        })
    }
}

pub fn traitify(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse2::<Args>(args).unwrap();
    let input = syn::parse2::<syn::ItemImpl>(input).unwrap();

    let trait_definition = {
        let trait_name = args.trait_name;
        let functions = input.items.iter().filter_map(|item| match item {
            syn::ImplItem::Fn(function) => {
                if matches!(function.vis, syn::Visibility::Public(_))
                    && function.sig.constness.is_none()
                    && function.sig.abi.is_none()
                {
                    let mut function_signature = function.sig.clone();

                    // No &self makes the trait not object safe. So add a where clause making it `Self: Sized`.
                    if function.sig.receiver().is_none() {
                        let function_where_clause = function_signature.generics.make_where_clause();
                        function_where_clause
                            .predicates
                            .push(syn::parse_quote!(Self: Sized));
                    }
                    Some(function_signature)
                } else {
                    None
                }
            }
            _ => None,
        });
        let trait_generics = input.generics.params.iter().filter(|param| match param {
            syn::GenericParam::Type(t) => !args.dyn_generics.contains(&t.ident.to_string()),
            syn::GenericParam::Const(t) => !args.dyn_generics.contains(&t.ident.to_string()),
            _ => true,
        });

        let mut trait_where = input.generics.clone();
        let trait_where =
            trait_where
                .make_where_clause()
                .predicates
                .iter()
                .filter(|pred| match pred {
                    syn::WherePredicate::Lifetime(_) => true,
                    syn::WherePredicate::Type(t) => match &t.bounded_ty {
                        syn::Type::Path(p) => !args
                            .dyn_generics
                            .contains(&p.path.to_token_stream().to_string()),
                        _ => todo!(),
                    },
                    _ => true,
                });

        quote!(
            pub trait #trait_name<#(#trait_generics,)*> where #(#trait_where,)* {
                #(#functions;)*
            }
        )
    };

    quote!(
        #input
        #trait_definition
    )
}
