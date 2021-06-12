use proc_macro::TokenStream;

use proc_macro2::TokenTree::Literal;
use quote::quote;
use syn::{DeriveInput, Expr, ItemEnum, ItemStruct, parse_macro_input, Token};
use syn::parse::{Parse, ParseStream};

struct MyParams(syn::LitInt, syn::Ident);

impl Parse for MyParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let type1 = content.parse()?;
        content.parse::<Token![,]>()?;
        let type2 = content.parse()?;
        Ok(MyParams(type1, type2))
    }
}

// https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros

#[proc_macro_derive(Packet, attributes(packet))]
pub fn packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let packe_attr = input.attrs
        .iter().find(|attr| attr.path.is_ident("packet"))
        .expect("need packet attibute");


    let name = input.ident;

    let MyParams(id, kind) = syn::parse(packe_attr.tokens.clone().into()).expect("Invalid attributes!");

    let expanded = quote! {

        impl packets::types::Packet for #name {
            const ID: u32 = #id;
            const STATE: packets::types::PacketState = packets::types::PacketState::#kind;
        }
    };


    TokenStream::from(expanded)
}

#[proc_macro_derive(Writable)]
pub fn writable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let name = input.ident;

    let idents = input.fields.iter().map(|x| {
        x.ident.as_ref().unwrap()
    });

    let expanded = quote! {
        impl packets::write::ByteWritable for #name {
            fn write_to_bytes(self, writer: &mut packets::write::ByteWriter) {
                writer.#(write(self.#idents)).*;
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Readable)]
pub fn readable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let name = input.ident;

    let idents = input.fields.iter().map(|x| {
        x.ident.as_ref().unwrap()
    });

    let expanded = quote! {
        impl packets::read::ByteReadable for #name {
            fn read_from_bytes(byte_reader: &mut packets::read::ByteReader) -> Self {
                #name {
                    #(#idents: byte_reader.read()),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumWritable)]
pub fn enum_writable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let name = input.ident;

    let expanded = quote! {
        impl ByteWritable for #name {
            fn write_to_bytes(self, writer: &mut ByteWriter) {
                let v = self as i32;
                writer.write(VarInt(v));
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumReadable)]
pub fn enum_readable_count(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let name = input.ident;

    // let mut discriminants = input.variants.iter().map(|x|x.discriminant.clone().unwrap().1);

    let mut idents = input.variants.iter().map(|x| x.ident.clone());
    let mut discriminants = input.variants.iter()
        .enumerate()
        .map(|(a, b)| proc_macro2::Literal::i32_unsuffixed(a as i32));

    let expanded = quote! {
        impl packets::read::ByteReadable for #name {
            fn read_from_bytes(byte_reader: &mut packets::read::ByteReader) -> Self {
                let VarInt(inner) = byte_reader.read();

                let res = match inner {
                    #(#discriminants => Some(#name::#idents)),*,
                    _ => None
                };

                res.unwrap()
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(AdtReadable)]
pub fn enum_readable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let name = input.ident;

    // let mut discriminants = input.variants.iter().map(|x|x.discriminant.clone().unwrap().1);


    let mut discriminants = input.variants.iter()
        .enumerate()
        .map(|(a, b)| proc_macro2::Literal::i32_unsuffixed(a as i32));

    let mut variants_ts = Vec::new();
    for variant in input.variants.clone() {
        let var_ident = variant.ident;
        let var_fields = variant.fields.iter().map(|x| x.ident.clone());
        let variant_ts = quote! {
            #name::#var_ident {
                #(#var_fields: byte_reader.read()),*
            }
        };
        variants_ts.push(variant_ts);
    }


    let expanded = quote! {
        impl packets::read::ByteReadable for #name {
            fn read_from_bytes(byte_reader: &mut packets::read::ByteReader) -> Self {
                let VarInt(inner) = byte_reader.read();

                let res = match inner {
                    #(#discriminants => Some(#variants_ts)),*,
                    _ => None
                };

                res.unwrap()
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(AdtWritable)]
pub fn adt_writable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let name = input.ident;

    // let mut discriminants = input.variants.iter().map(|x|x.discriminant.clone().unwrap().1);

    let idents: Vec<_> = input.variants.clone().iter().map(|x|x.ident.clone()).collect();

    let mut discriminants = input.variants.iter()
        .enumerate()
        .map(|(a, b)| proc_macro2::Literal::i32_unsuffixed(a as i32));

    let mut variants_ts = Vec::new();
    for variant in input.variants.clone() {
        let var_ident = variant.ident;
        let var_fields: Vec<_> = variant.fields.iter().map(|x| x.ident.clone().unwrap().clone()).collect();
        let variant_ts = quote! {
            #name::#var_ident { #(#var_fields),* }=> {
                #(writer.write(#var_fields));*;
            }
        };
        variants_ts.push(variant_ts);
    }


    let expanded = quote! {
        impl packets::write::ByteWritable for #name {
            fn write_to_bytes(self, writer: &mut packets::write::ByteWriter) {

                let id = match self {
                    #(#name::#idents{..} => #discriminants),*,
                };

                let id = VarInt(id);

                writer.write(id);

                match self {
                    #(#variants_ts),*,
                };

            }
        }
    };

    TokenStream::from(expanded)
}
