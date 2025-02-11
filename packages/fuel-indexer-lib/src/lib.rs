//! # fuel_indexer_lib
//!
//! A collection of utilities used by the various `fuel-indexer-*` crates.

#![deny(unused_crate_dependencies)]
pub mod config;
pub mod defaults;
pub mod graphql;
pub mod manifest;
pub mod utils;

use proc_macro2::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};

/// Max size of Postgres array types.
pub const MAX_ARRAY_LENGTH: usize = 2500;

/// The source of execution for the indexer.
#[derive(Default, Clone, Debug)]
pub enum ExecutionSource {
    /// The indexer is being executed as a standalone binary.
    Native,

    /// The indexer is being executed in a WASM runtime.
    #[default]
    Wasm,
}

impl ExecutionSource {
    pub fn async_awaitness(&self) -> (TokenStream, TokenStream) {
        match self {
            Self::Native => (quote! {async}, quote! {.await}),
            Self::Wasm => (quote! {}, quote! {}),
        }
    }
}

/// Derive a type ID from a namespace and given abstraction name.
pub fn type_id(namespace: &str, name: &str) -> i64 {
    // IMPORTANT: https://github.com/launchbadge/sqlx/issues/499
    let mut bytes = [0u8; 8];
    let digest = Sha256::digest(format!("{name}:{namespace}").as_bytes());
    bytes[..8].copy_from_slice(&digest[..8]);
    i64::from_be_bytes(bytes)
}

/// Return a fully qualified indexer namespace.
pub fn fully_qualified_namespace(namespace: &str, identifier: &str) -> String {
    format!("{}_{}", namespace, identifier)
}

/// Return the name of the join table for the given entities.
pub fn join_table_name(a: &str, b: &str) -> String {
    format!("{}s_{}s", a, b)
}

/// Return the name of each TypeDefinition in the join table.
pub fn join_table_typedefs_name(join_table_name: &str) -> (String, String) {
    let mut parts = join_table_name.split('_');
    let a = parts.next().unwrap();
    let b = parts.next().unwrap();

    // Trim the plural 's' from the end of the TypeDefinition name.
    (a[0..a.len() - 1].to_string(), b[0..b.len() - 1].to_string())
}
