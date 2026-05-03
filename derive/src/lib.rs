use quote::{ToTokens, quote};
use syn::{DeriveInput, Type, parse_macro_input};

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
        impl mimiq::graphics::Vertex for #ident {
            const LAYOUT: &'static [mimiq::graphics::VertexField] = &[
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
        impl mimiq::graphics::UniformBlock for #ident {
            const FIELDS: &'static [mimiq::graphics::UniformField] = &[
                #(#field_invocations),*
            ];
        }
    };

    output.into()
}

#[proc_macro_derive(ImagesUniformBlock)]
pub fn derive_images_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let DeriveInput { ident, data, .. } = input;
    let syn::Data::Struct(fields) = data else {
        return quote! {
            compile_error!("Only structs are supported");
        }
        .into();
    };

    let field_names = fields
        .fields
        .iter()
        .map(|field| match &field.ident {
            None => quote! {
                compile_error!("Unit structs are not supported");
            },
            Some(field_name) => field_name.into_token_stream(),
        })
        .collect::<Vec<_>>();
    let field_invocations = fields
        .fields
        .iter()
        .map(|field| match &field.ident {
            None => quote! {
                compile_error!("Unit structs are not supported");
            },
            Some(field_name) => match &field.ty {
                Type::Reference(ty_ref) => {
                    let field_type = &ty_ref.elem;
                    quote! {
                        mimiq::image_uniform_of!(#field_type, #field_name)
                    }
                }
                _ => quote! {
                    compile_error!("Only reference type fields are allowed");
                },
            },
        })
        .collect::<Vec<_>>();
    let slots = 0..(field_invocations.len() as u32);

    let output = quote! {
        impl mimiq::graphics::ImagesUniformBlock for #ident<'_> {
            type Borrow<'a> = &'a #ident<'a>;

            const FIELDS: &'static [mimiq::graphics::ImageUniformField] = &[
                #(#field_invocations),*
            ];

            fn bind(borrow: Self::Borrow<'_>) {
                use mimiq::graphics::ImageUniformVal;
                #( borrow.#field_names.bind(#slots) );*
            }
        }
    };

    output.into()
}
