use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, Ident, ItemStruct};

#[derive(Debug)]
struct WriteableField {
    ident: Ident,
    is_tag: bool,
    is_ignore: bool,
}

impl From<Field> for WriteableField {
    fn from(field: Field) -> WriteableField {
        let ident = field.ident.expect("fields without ident are not supported");

        let check_influx_aware = |attr: &syn::Attribute| -> bool {
            attr.path
                .segments
                .iter()
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default()
                == "influxdb"
        };

        let check_for_attr = |token_tree, ident_cmp: &str| -> bool {
            match token_tree {
                TokenTree::Group(group) => group
                    .stream()
                    .into_iter()
                    .next()
                    .map(|token_tree| match token_tree {
                        TokenTree::Ident(ident) => ident == ident_cmp,
                        _ => false,
                    })
                    .unwrap(),
                _ => false,
            }
        };

        let is_ignore = field.attrs.iter().any(|attr| {
            if !check_influx_aware(attr) {
                return false;
            }

            attr.tokens
                .clone()
                .into_iter()
                .next()
                .map(|token_tree| check_for_attr(token_tree, "ignore"))
                .unwrap()
        });

        let is_tag = field.attrs.iter().any(|attr| {
            if !check_influx_aware(attr) {
                return false;
            }
            attr.tokens
                .clone()
                .into_iter()
                .next()
                .map(|token_tree| check_for_attr(token_tree, "tag"))
                .unwrap()
        });

        WriteableField {
            ident,
            is_tag,
            is_ignore,
        }
    }
}

pub fn expand_writeable(tokens: TokenStream) -> TokenStream {
    let krate = super::krate();
    let input = parse_macro_input!(tokens as ItemStruct);
    let ident = input.ident;
    let generics = input.generics;

    let time_field = format_ident!("time");
    #[allow(clippy::cmp_owned)] // that's not how idents work clippy
    let fields: Vec<TokenStream2> = match input.fields {
        Fields::Named(fields) => fields
            .named
            .into_iter()
            .map(WriteableField::from)
            .filter(|field| !field.is_ignore)
            .filter(|field| field.ident.to_string() != time_field.to_string())
            .map(|field| {
                let ident = field.ident;
                #[allow(clippy::match_bool)]
                match field.is_tag {
                    true => quote!(query.add_tag(stringify!(#ident), self.#ident)),
                    false => quote!(query.add_field(stringify!(#ident), self.#ident)),
                }
            })
            .collect(),
        _ => panic!("a struct without named fields is not supported"),
    };

    let output = quote! {
        impl #generics #krate::InfluxDbWriteable for #ident #generics
        {
            fn into_query<I: Into<String>>(self, name : I) -> #krate::WriteQuery
            {
                let timestamp : #krate::Timestamp = self.#time_field.into();
                let mut query = timestamp.into_query(name);
                #(
                    query = #fields;
                )*
                query
            }
        }
    };
    output.into()
}
