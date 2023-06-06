use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, ImplItemFn, PathArguments, PathSegment,
    TypePath,
};

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
    let args = syn::parse2::<Args>(args).expect("Parsing args");
    let input = syn::parse2::<syn::ItemImpl>(input).unwrap();

    let functions = input
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(function) => {
                if matches!(function.vis, syn::Visibility::Public(_))
                    && function.sig.constness.is_none()
                    && function.sig.abi.is_none()
                    // No arguments may of a generic type we're dyn over
                    && function.sig.inputs.iter().all(|arg| match arg {
                        syn::FnArg::Receiver(_) => true,
                        syn::FnArg::Typed(t) => !args.dyn_generics.contains(&t.ty.to_token_stream().to_string()),
                    })
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
        })
        .collect::<Vec<_>>();

    let trait_generics = input
        .generics
        .params
        .iter()
        .filter(|param| match param {
            syn::GenericParam::Type(t) => !args.dyn_generics.contains(&t.ident.to_string()),
            syn::GenericParam::Const(t) => !args.dyn_generics.contains(&t.ident.to_string()),
            _ => true,
        })
        .collect::<Vec<_>>();

    let trait_definition = {
        let trait_name = args.trait_name.clone();

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

    let trait_impl = {
        // We're gonna take the original impl, strip out all functions, make it implement the exact functions of the trait
        let mut trait_impl = input.clone();
        trait_impl.items.clear();

        trait_impl.attrs.clear();

        let trait_generic_arguments = if trait_generics.is_empty() {
            PathArguments::None
        } else {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: syn::Token![<](Span::call_site()),
                args: trait_generics
                    .iter()
                    .map(|param| match param {
                        syn::GenericParam::Lifetime(lt) => {
                            GenericArgument::Lifetime(lt.lifetime.clone())
                        }
                        syn::GenericParam::Type(t) => {
                            GenericArgument::Type(syn::Type::Path(TypePath {
                                qself: None,
                                path: PathSegment {
                                    ident: t.ident.clone(),
                                    arguments: Default::default(),
                                }
                                .into(),
                            }))
                        }
                        syn::GenericParam::Const(c) => {
                            GenericArgument::Type(syn::Type::Path(TypePath {
                                qself: None,
                                path: PathSegment {
                                    ident: c.ident.clone(),
                                    arguments: Default::default(),
                                }
                                .into(),
                            }))
                        }
                    })
                    .collect(),
                gt_token: syn::Token![>](Span::call_site()),
            })
        };
        trait_impl.trait_ = Some((
            None,
            PathSegment {
                ident: args.trait_name,
                arguments: trait_generic_arguments,
            }
            .into(),
            syn::token::For(Span::call_site()),
        ));

        trait_impl.items = functions
            .iter()
            .map(|signature| {
                let function_name = signature.ident.clone();
                let function_params = signature.inputs.iter().map(|arg| match arg {
                    syn::FnArg::Receiver(_) => quote!(self),
                    syn::FnArg::Typed(t) => t.pat.to_token_stream(),
                });

                syn::ImplItem::Fn(ImplItemFn {
                    attrs: Vec::new(),
                    vis: syn::Visibility::Inherited,
                    defaultness: None,
                    sig: signature.clone(),
                    block: syn::parse_quote!({
                        Self::#function_name(#(#function_params,)*)
                    }),
                })
            })
            .collect();

        trait_impl
    };

    quote!(
        #input

        #trait_definition

        #trait_impl
    )
}
