use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Data, DeriveInput, Field, Fields, Ident, Meta, Token,
};

#[derive(Debug)]
struct WriteableField {
    ident: Ident,
    is_time: bool,
    is_tag: bool,
    is_ignore: bool,
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(time);
    custom_keyword!(tag);
    custom_keyword!(ignore);
}

#[allow(dead_code)] // TODO do we need to store the keywords?
enum FieldAttr {
    Time(kw::time),
    Tag(kw::tag),
    Ignore(kw::ignore),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::time) {
            Ok(Self::Time(input.parse()?))
        } else if lookahead.peek(kw::tag) {
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
        let mut has_time_attr = false;
        let mut is_tag = false;
        let mut is_ignore = false;

        for attr in field.attrs {
            match attr.meta {
                Meta::List(list) if list.path.is_ident("influxdb") => {
                    for attr in syn::parse2::<FieldAttrs>(list.tokens)?.0 {
                        match attr {
                            FieldAttr::Time(_) => has_time_attr = true,
                            FieldAttr::Tag(_) => is_tag = true,
                            FieldAttr::Ignore(_) => is_ignore = true,
                        }
                    }
                }
                _ => {}
            }
        }

        if [has_time_attr, is_tag, is_ignore]
            .iter()
            .filter(|&&b| b)
            .count()
            > 1
        {
            panic!("only one of time, tag, or ignore can be used");
        }

        // A field is considered a time field if:
        // 1. It has the #[influxdb(time)] attribute, OR
        // 2. It's named "time" and doesn't have #[influxdb(ignore)]
        let is_time = has_time_attr || (ident == "time" && !is_ignore);

        Ok(WriteableField {
            ident,
            is_time,
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

    let writeable_fields: Vec<WriteableField> = match fields {
        Fields::Named(fields) => fields
            .named
            .into_iter()
            .map(WriteableField::try_from)
            .collect::<syn::Result<Vec<_>>>()?,
        _ => panic!("A struct without named fields is not supported!"),
    };

    // Find the time field
    let mut time_field = None;
    for wf in &writeable_fields {
        if wf.is_time {
            if time_field.is_some() {
                panic!("multiple time fields found!");
            }
            time_field = Some(wf.ident.clone());
        }
    }

    // There must be exactly one time field
    let time_field = time_field.expect("no time field found");

    // Generate field assignments (excluding time and ignored fields)
    let field_assignments = writeable_fields
        .into_iter()
        .filter_map(|wf| {
            if wf.is_ignore || wf.is_time {
                None
            } else {
                let ident = wf.ident;
                Some(match wf.is_tag {
                    true => quote!(query.add_tag(stringify!(#ident), self.#ident)),
                    false => quote!(query.add_field(stringify!(#ident), self.#ident)),
                })
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        impl #impl_generics ::influxdb::InfluxDbWriteable for #ident #ty_generics #where_clause {
            fn into_query<I: Into<String>>(self, name: I) -> ::influxdb::WriteQuery {
                let timestamp: ::influxdb::Timestamp = self.#time_field.into();
                let mut query = timestamp.into_query(name);
                #(
                    query = #field_assignments;
                )*
                query
            }
        }
    })
}
