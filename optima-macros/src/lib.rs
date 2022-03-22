#![crate_type = "proc-macro"]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Solution)]
pub fn solution_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    eprintln!("{:#?}", &ast);

    let name = &ast.ident;

    quote! {
        impl crate::base::Solution for #name {
            fn get_value(&self) -> f64{
                self.eval.value
            }
            fn get_eval(&self) -> &crate::base::Evaluation{
                &self.eval
            }
            fn get_eval_mut(&mut self) -> &mut crate::base::Evaluation{
                &mut self.eval
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn solution_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => match &mut struct_data.fields {
            syn::Fields::Named(fields) => {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! { eval: Evaluation })
                        .unwrap(),
                );
                return quote! {
                    #ast
                }
                .into();
            }
            _ => Error::new_spanned(&ast.ident, "Struct need to have at least empty body")
                .to_compile_error()
                .into(),
        },
        _ => Error::new_spanned(&ast.ident, "New fields can be added only to struct")
            .to_compile_error()
            .into(),
    }
}