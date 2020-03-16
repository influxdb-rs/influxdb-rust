use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, Ident, ItemStruct};

struct WriteableField {
    ident: Ident,
    is_tag: bool,
}

impl From<Field> for WriteableField {
    fn from(field: Field) -> WriteableField {
        let ident = field.ident.expect("fields without ident are not supported");
        let is_tag = field
            .attrs
            .iter()
            .filter(|attr| {
                attr.path
                    .segments
                    .iter()
                    .last()
                    .map(|seg| seg.ident.to_string())
                    .unwrap_or_default()
                    == "tag"
            })
            .nth(0)
            .is_some();
        WriteableField { ident, is_tag }
    }
}

pub fn expand_writeable(tokens: TokenStream) -> TokenStream {
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
            .filter(|field| field.ident.to_string() != time_field.to_string())
            .map(|field| {
                let ident = field.ident;
                #[allow(clippy::match_bool)]
                match field.is_tag {
                    true => quote!(query.add_field(stringify!(#ident), self.#ident)),
                    false => quote!(query.add_field(stringify!(#ident), self.#ident)),
                }
            })
            .collect(),
        _ => panic!("a struct without named fields is not supported"),
    };

    let output = quote! {
        impl #generics ::influxdb::InfluxDbWriteable for #ident #generics
        {
            fn into_query<I: Into<String>>(self, name : I) -> ::influxdb::WriteQuery
            {
                let timestamp : ::influxdb::Timestamp = self.#time_field.into();
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
