use std::str::FromStr;

use lazy_static::lazy_static;
use proc_macro2::Ident;
use proc_macro_error::{abort, abort_call_site};
use regex::{Captures, Regex};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Comma, FnArg, LitStr, PatIdent, Token, Type,
};

use crate::{ext::ResolvedArg, path::PathOperation};

use super::{
    ArgumentResolver, PathOperationResolver, PathOperations, PathResolver, ResolvedOperation,
    ResolvedPath,
};

impl ArgumentResolver for PathOperations {
    fn resolve_path_arguments(
        fn_args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        resolved_args: Option<Vec<ResolvedArg>>,
    ) -> Option<Vec<super::Argument<'_>>> {
        dbg!(&fn_args);

        resolved_args.map(|args| {
            let (anonymous_args, named_args): (Vec<ResolvedArg>, Vec<ResolvedArg>) =
                args.into_iter().partition(|arg| {
                    matches!(arg, ResolvedArg::Path(path) if path.contains("arg"))
                        || matches!(arg, ResolvedArg::Query(query) if query.contains("arg"))
                });

            // TODO
        });
        // TODO
        None
    }
}

pub struct Arg<'a> {
    pub name: &'a Ident,
    pub ty: &'a Ident,
    pub is_array: bool,
}

impl PathOperations {
    fn get_argument_name_and_type(fn_args: Punctuated<FnArg, Comma>) -> impl Iterator {
        // TODO

        fn_args.into_iter().map(|arg| match arg {
            FnArg::Typed(pat_type) => {
                let ident = match pat_type.pat.as_ref() {
                    syn::Pat::Ident(pat) => &pat.ident,
                    _ => abort_call_site!("unexpected Pat, expected Pat::Ident"),
                };

                let i = Self::get_type_ident(pat_type.ty.as_ref());

                // match pat_type.ty.as_ref() {
                //     Type::Path(path) => &path.path.segments.first().unwrap().ident,
                //     Type::Reference(reference) => reference.elem,
                //     _ => abort_call_site!(
                //         "unexpected pat type, expected one of: Type::Path, Type::Reference"
                //     ),
                // };

                // pat_type.ty

                // vec![]
                // match pat_type.pat.as_ref() {
                //     Pat::Ident(pat_ident) => pat_ident.ident;
                // }
            }
            _ => abort_call_site!("unexpected FnArg, expected FnArg::Typed"),
        })
    }

    fn get_type_ident(ty: &Type) -> &Ident {
        match ty {
            Type::Path(path) => {
                let segment = &path.path.segments.first().unwrap();

                &segment.ident
                // if segment.arguments.is_empty() {
                //     &segment.ident
                // } else {
                //     // TODO handle segment arguments get type
                //     // &segment.ident
                // }
            }
            Type::Reference(reference) => Self::get_type_ident(reference.elem.as_ref()),
            _ => abort_call_site!(
                "unexpected pat type, expected one of: Type::Path, Type::Reference"
            ),
        }
    }
}

impl PathOperationResolver for PathOperations {
    fn resolve_operation(ast_fn: &syn::ItemFn) -> Option<super::ResolvedOperation> {
        ast_fn.attrs.iter().find_map(|attribute| {
            if is_valid_route_type(attribute.path.get_ident()) {
                let Path(path, operation) = match attribute.parse_args::<Path>() {
                    Ok(path) => path,
                    Err(error) => abort!(
                        error.span(),
                        "parse path of path operation attribute: {}",
                        error
                    ),
                };

                if let Some(operation) = operation {
                    Some(ResolvedOperation {
                        path_operation: PathOperation::from_str(&operation).unwrap(),
                        path,
                    })
                } else {
                    Some(ResolvedOperation {
                        path_operation: PathOperation::from_ident(
                            attribute.path.get_ident().unwrap(),
                        ),
                        path,
                    })
                }
            } else {
                None
            }
        })
    }
}

struct Path(String, Option<String>);

impl Parse for Path {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (path, operation) = if input.peek(syn::Ident) {
            // expect format (GET, uri = "url...")
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![,]>()?;
            input.parse::<Ident>()?; // explisitly 'uri'
            input.parse::<Token![=]>()?;

            (
                input.parse::<LitStr>()?.value(),
                Some(ident.to_string().to_lowercase()),
            )
        } else {
            // expect format ("url...")

            (input.parse::<LitStr>()?.value(), None)
        };

        input.step(|cursor| {
            let mut rest = *cursor;
            // ignore rest of the tokens from actix_web path attribute macro
            while let Some((tt, next)) = rest.token_tree() {
                rest = next;
            }
            Ok(((), rest))
        });

        Ok(Self(path, operation))
    }
}

#[inline]
fn is_valid_route_type(ident: Option<&Ident>) -> bool {
    matches!(ident, Some(operation) if ["get", "post", "put", "delete", "head", "options", "patch", "route"]
        .iter().any(|expected_operation| operation == expected_operation))
}

impl PathResolver for PathOperations {
    fn resolve_path(path: &Option<String>) -> Option<ResolvedPath> {
        path.as_ref().map(|whole_path| {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"<[a-zA-Z0-9_][^<>]*>").unwrap();
            }

            whole_path
                .split_once('?')
                .or(Some((&*whole_path, "")))
                .map(|(path, query)| {
                    let mut names =
                        Vec::<ResolvedArg>::with_capacity(RE.find_iter(whole_path).count());
                    let mut underscore_count = 0;

                    let mut format_arg =
                        |captures: &Captures, resolved_arg_op: fn(String) -> ResolvedArg| {
                            let mut capture = &captures[0];
                            let arg = capture
                                .replace("..", "")
                                .replace('<', "{")
                                .replace('>', "}");

                            if arg == "_" {
                                names.push(resolved_arg_op(format!("arg{underscore_count}")));
                                underscore_count += 1;
                            } else {
                                names.push(resolved_arg_op(String::from(&arg[1..arg.len() - 1])))
                            }

                            arg
                        };

                    let path = RE.replace_all(path, |captures: &Captures| {
                        format_arg(captures, ResolvedArg::Path)
                    });

                    let query = if !query.is_empty() {
                        Some(RE.replace_all(query, |captures: &Captures| {
                            format_arg(captures, ResolvedArg::Query)
                        }))
                    } else {
                        None
                    };

                    let path = [Some(path), query]
                        .into_iter()
                        .filter(Option::is_some)
                        .flatten()
                        .collect::<Vec<_>>()
                        .join("?");

                    ResolvedPath { args: names, path }
                })
                .unwrap()
        })
    }
}

// [
//     Typed(
//         PatType {
//             attrs: [],
//             pat: Ident(
//                 PatIdent {
//                     attrs: [],
//                     by_ref: None,
//                     mutability: None,
//                     ident: Ident {
//                         ident: "id",
//                         span: #0 bytes(1105..1107),
//                     },
//                     subpat: None,
//                 },
//             ),
//             colon_token: Colon,
//             ty: Path(
//                 TypePath {
//                     qself: None,
//                     path: Path {
//                         leading_colon: None,
//                         segments: [
//                             PathSegment {
//                                 ident: Ident {
//                                     ident: "i32",
//                                     span: #0 bytes(1109..1112),
//                                 },
//                                 arguments: None,
//                             },
//                         ],
//                     },
//                 },
//             ),
//         },
//     ),
//     Comma,
//     Typed(
//         PatType {
//             attrs: [],
//             pat: Ident(
//                 PatIdent {
//                     attrs: [],
//                     by_ref: None,
//                     mutability: None,
//                     ident: Ident {
//                         ident: "name",
//                         span: #0 bytes(1114..1118),
//                     },
//                     subpat: None,
//                 },
//             ),
//             colon_token: Colon,
//             ty: Reference(
//                 TypeReference {
//                     and_token: And,
//                     lifetime: None,
//                     mutability: None,
//                     elem: Path(
//                         TypePath {
//                             qself: None,
//                             path: Path {
//                                 leading_colon: None,
//                                 segments: [
//                                     PathSegment {
//                                         ident: Ident {
//                                             ident: "str",
//                                             span: #0 bytes(1121..1124),
//                                         },
//                                         arguments: None,
//                                     },
//                                 ],
//                             },
//                         },
//                     ),
//                 },
//             ),
//         },
//     ),
//     Comma,
//     Typed(
//         PatType {
//             attrs: [],
//             pat: Ident(
//                 PatIdent {
//                     attrs: [],
//                     by_ref: None,
//                     mutability: None,
//                     ident: Ident {
//                         ident: "colors",
//                         span: #0 bytes(1126..1132),
//                     },
//                     subpat: None,
//                 },
//             ),
//             colon_token: Colon,
//             ty: Path(
//                 TypePath {
//                     qself: None,
//                     path: Path {
//                         leading_colon: None,
//                         segments: [
//                             PathSegment {
//                                 ident: Ident {
//                                     ident: "Vec",
//                                     span: #0 bytes(1134..1137),
//                                 },
//                                 arguments: AngleBracketed(
//                                     AngleBracketedGenericArguments {
//                                         colon2_token: None,
//                                         lt_token: Lt,
//                                         args: [
//                                             Type(
//                                                 Reference(
//                                                     TypeReference {
//                                                         and_token: And,
//                                                         lifetime: None,
//                                                         mutability: None,
//                                                         elem: Path(
//                                                             TypePath {
//                                                                 qself: None,
//                                                                 path: Path {
//                                                                     leading_colon: None,
//                                                                     segments: [
//                                                                         PathSegment {
//                                                                             ident: Ident {
//                                                                                 ident: "str",
//                                                                                 span: #0 bytes(1139..1142),
//                                                                             },
//                                                                             arguments: None,
//                                                                         },
//                                                                     ],
//                                                                 },
//                                                             },
//                                                         ),
//                                                     },
//                                                 ),
//                                             ),
//                                         ],
//                                         gt_token: Gt,
//                                     },
//                                 ),
//                             },
//                         ],
//                     },
//                 },
//             ),
//         },
//     ),
// ]
