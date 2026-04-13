use quote::quote;
use syn::{DeriveInput, parse_macro_input};

extern crate proc_macro;

#[proc_macro_derive(Vertex)]
pub fn derive_vertex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let DeriveInput { ident, data, .. } = input;
    let syn::Data::Struct(fields) = data else {
        return quote! {
            compile_error!("Only structs are supported");
        }
        .into();
    };

    let field_invocations = fields
        .fields
        .into_iter()
        .map(|field| match field.ident {
            None => quote! {
                compile_error!("Unit structs are not supported");
            },
            Some(field_name) => quote! {
                mimiq::attribute_of!(#ident, #field_name)
            },
        })
        .collect::<Vec<_>>();

    let output = quote! {
        impl mimiq::Vertex for #ident {
            const LAYOUT: &'static [mimiq::VertexField] = &[
                #(#field_invocations),*
            ];
        }
    };

    output.into()
}

#[proc_macro_derive(UniformBlock)]
pub fn derive_uniform_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let DeriveInput { ident, data, .. } = input;
    let syn::Data::Struct(fields) = data else {
        return quote! {
            compile_error!("Only structs are supported");
        }
        .into();
    };

    let field_invocations = fields
        .fields
        .into_iter()
        .map(|field| match field.ident {
            None => quote! {
                compile_error!("Unit structs are not supported");
            },
            Some(field_name) => quote! {
                mimiq::uniform_of!(#ident, #field_name)
            },
        })
        .collect::<Vec<_>>();

    let output = quote! {
        impl mimiq::UniformBlock for #ident {
            const FIELDS: &'static [mimiq::UniformField] = &[
                #(#field_invocations),*
            ];
        }
    };

    output.into()
}
