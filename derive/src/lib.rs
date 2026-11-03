use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod backend;
mod frontend;
mod model;

#[proc_macro_derive(FromEnv, attributes(env))]
pub fn derive_from_env(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    frontend::parse(&ast)
        .map(|container| backend::expand(&container))
        .unwrap_or_else(|e| e.write_errors())
        .into()
}
