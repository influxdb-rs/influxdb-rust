use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::convert::TryFrom;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Data, DeriveInput, Field, Fields, Ident, Meta, Token,
};

#[derive(Debug)]
struct WriteableField {
    ident: Ident,
    is_tag: bool,
    is_ignore: bool,
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(tag);
    custom_keyword!(ignore);
}

enum FieldAttr {
    Tag(kw::tag),
    Ignore(kw::ignore),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::tag) {
            Ok(Self::Tag(input.parse()?))
        } else if lookahead.peek(kw::ignore) {
            Ok(Self::Ignore(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

struct FieldAttrs(Punctuated<FieldAttr, Token![,]>);

impl Parse for FieldAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self(Punctuated::parse_terminated(input)?))
    }
}

impl TryFrom<Field> for WriteableField {
    type Error = syn::Error;

    fn try_from(field: Field) -> syn::Result<WriteableField> {
        let ident = field.ident.expect("fields without ident are not supported");
        let mut is_tag = false;
        let mut is_ignore = false;

        for attr in field.attrs {
            match attr.meta {
                Meta::List(list) if list.path.is_ident("influxdb") => {
                    for attr in syn::parse2::<FieldAttrs>(list.tokens)?.0 {
                        match attr {
                            FieldAttr::Tag(_) => is_tag = true,
                            FieldAttr::Ignore(_) => is_ignore = true,
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(WriteableField {
            ident,
            is_tag,
            is_ignore,
        })
    }
}

pub fn expand_writeable(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(strukt) => strukt.fields,
        Data::Enum(inum) => {
            return Err(syn::Error::new(
                inum.enum_token.span,
                "#[derive(InfluxDbWriteable)] can only be used on structs",
            ))
        }
        Data::Union(cdu) => {
            return Err(syn::Error::new(
                cdu.union_token.span,
                "#[derive(InfluxDbWriteable)] can only be used on structs",
            ))
        }
    };

    let time_field = format_ident!("time");
    let time_field_str = time_field.to_string();
    #[allow(clippy::cmp_owned)] // that's not how idents work clippy
    let fields = match fields {
        Fields::Named(fields) => fields
            .named
            .into_iter()
            .filter_map(|f| {
                WriteableField::try_from(f)
                    .map(|wf| {
                        if !wf.is_ignore && wf.ident.to_string() != time_field_str {
                            let ident = wf.ident;
                            Some(match wf.is_tag {
                                true => quote!(query.add_tag(stringify!(#ident), self.#ident)),
                                false => quote!(query.add_field(stringify!(#ident), self.#ident)),
                            })
                        } else {
                            None
                        }
                    })
                    .transpose()
            })
            .collect::<syn::Result<Vec<_>>>()?,
        _ => panic!("a struct without named fields is not supported"),
    };

    Ok(quote! {
        impl #impl_generics ::influxdb::InfluxDbWriteable for #ident #ty_generics #where_clause {
            fn into_query<I: Into<String>>(self, name: I) -> ::influxdb::WriteQuery {
                let timestamp: ::influxdb::Timestamp = self.#time_field.into();
                let mut query = timestamp.into_query(name);
                #(
                    query = #fields;
                )*
                query
            }
        }
    })
}
