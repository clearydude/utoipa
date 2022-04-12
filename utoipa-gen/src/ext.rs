#![allow(unused)]
use proc_macro2::Ident;
use syn::{punctuated::Punctuated, token::Comma, Attribute, FnArg, ItemFn};

use crate::path::PathOperation;

#[cfg(feature = "actix_extras")]
pub mod actix;
#[cfg(feature = "rocket_extras")]
pub mod rocket;

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Argument<'a> {
    pub name: Option<&'a str>,
    pub argument_in: ArgumentIn,
    pub ident: &'a Ident,
}

impl Argument<'_> {
    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
pub enum ArgumentIn {
    Path,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResolvedPath {
    pub path: String,
    pub args: Vec<ResolvedArg>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ResolvedArg {
    Path(String),
    Query(String),
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResolvedOperation {
    pub path_operation: PathOperation,
    pub path: String,
}

pub trait ArgumentResolver {
    fn resolve_path_arguments(
        _: &Punctuated<FnArg, Comma>,
        _: Option<Vec<ResolvedArg>>,
    ) -> Option<Vec<Argument<'_>>> {
        None
    }
}

pub trait PathResolver {
    fn resolve_path(_: &Option<String>) -> Option<ResolvedPath> {
        None
    }
}

pub trait PathOperationResolver {
    fn resolve_operation(_: &ItemFn) -> Option<ResolvedOperation> {
        None
    }
}

pub struct PathOperations;

// #[cfg(not(feature = "actix_extras"))]
#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl ArgumentResolver for PathOperations {}
// #[cfg(not(feature = "actix_extras"))]
#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl PathResolver for PathOperations {}
// #[cfg(all(not(feature = "actix_extras"), not(feature = "rocket_extras")))]
#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl PathOperationResolver for PathOperations {}
