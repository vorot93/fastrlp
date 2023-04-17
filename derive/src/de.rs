use alloc::vec::Vec;
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_decodable(ast: &syn::DeriveInput) -> TokenStream {
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(Decodable)] is only defined for structs.");
    };

    let stmts: Vec<_> = body
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| decodable_field(i, field))
        .collect();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_block = quote! {
        impl #impl_generics fastrlp::Decodable for #name #ty_generics #where_clause {
            fn decode(mut buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
                let b = &mut &**buf;
                let rlp_head = fastrlp::Header::decode(b)?;

                if !rlp_head.list {
                    return ::core::result::Result::Err(fastrlp::DecodeError::UnexpectedString);
                }

                let started_len = b.len();
                let this = Self {
                    #(#stmts)*
                };

                let consumed = started_len - b.len();
                if consumed != rlp_head.payload_length {
                    return ::core::result::Result::Err(fastrlp::DecodeError::ListLengthMismatch {
                        expected: rlp_head.payload_length,
                        got: consumed,
                    });
                }

                *buf = *b;

                ::core::result::Result::Ok(this)
            }
        }
    };

    quote! {
        const _: () = {
            extern crate fastrlp;
            #impl_block
        };
    }
}

pub fn impl_decodable_wrapper(ast: &syn::DeriveInput) -> TokenStream {
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(EncodableWrapper)] is only defined for structs.");
    };

    assert_eq!(
        body.fields.iter().count(),
        1,
        "#[derive(EncodableWrapper)] is only defined for structs with one field."
    );

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_block = quote! {
        impl #impl_generics fastrlp::Decodable for #name #ty_generics #where_clause {
            fn decode(buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
                ::core::result::Result::Ok(Self(fastrlp::Decodable::decode(buf)?))
            }
        }
    };

    quote! {
        const _: () = {
            extern crate fastrlp;
            #impl_block
        };
    }
}

fn decodable_field(index: usize, field: &syn::Field) -> TokenStream {
    let id = if let Some(ident) = &field.ident {
        quote! { #ident }
    } else {
        let index = syn::Index::from(index);
        quote! { #index }
    };

    quote! { #id: fastrlp::Decodable::decode(b)?, }
}
