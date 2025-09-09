use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    AngleBracketedGenericArguments, Data, DeriveInput, Field, Fields, GenericArgument, Ident,
    Lifetime, Meta, PathArguments, PredicateType, Token, Type, TypeParamBound, WhereClause,
    WherePredicate,
};
use syn_path::type_path;

#[derive(Debug)]
struct WriteableField {
    ident: Ident,
    ty: Type,
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
        let ty = field.ty;
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
            ty,
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
    let mut time_field_ty = None;
    for wf in &writeable_fields {
        if wf.is_time {
            if time_field.is_some() {
                panic!("multiple time fields found!");
            }
            time_field = Some(wf.ident.clone());
            time_field_ty = Some(wf.ty.clone());
        }
    }

    // There must be exactly one time field
    let time_field = time_field.expect("no time field found");
    let time_field_ty = time_field_ty.unwrap();

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

    // Add a necessary where clause
    let mut where_clause = where_clause.cloned().unwrap_or(WhereClause {
        where_token: Default::default(),
        predicates: Punctuated::new(),
    });
    let mut err_ty = type_path!(<::influxdb::Timestamp as ::core::convert::TryFrom>::Error);
    err_ty
        .path
        .segments
        .iter_mut()
        .nth(err_ty.qself.as_ref().unwrap().position - 1)
        .unwrap()
        .arguments = PathArguments::AngleBracketed(AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Default::default(),
        args: [GenericArgument::Type(time_field_ty.clone())]
            .into_iter()
            .collect(),
        gt_token: Default::default(),
    });
    where_clause
        .predicates
        .push(WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: Type::Path(err_ty),
            colon_token: Default::default(),
            bounds: [TypeParamBound::Lifetime(Lifetime {
                apostrophe: Span::call_site(),
                ident: format_ident!("static"),
            })]
            .into_iter()
            .collect(),
        }));

    // Assemble the rest of the code
    Ok(quote! {
        const _: () = {
            mod __influxdb_private {
                use ::influxdb::{InfluxDbWriteable, Timestamp};
                use ::core::fmt::{self, Debug, Display, Formatter, Write as _};

                pub enum Error<T>
                where
                    Timestamp: TryFrom<T>
                {
                    TimestampError(<Timestamp as TryFrom<T>>::Error),
                    QueryError(<Timestamp as InfluxDbWriteable>::Error)
                }

                impl<T> Clone for Error<T>
                where
                    Timestamp: TryFrom<T>,
                    <Timestamp as TryFrom<T>>::Error: Clone
                {
                    fn clone(&self) -> Self {
                        match self {
                            Self::TimestampError(err) => Self::TimestampError(err.clone()),
                            Self::QueryError(err) => Self::QueryError(err.clone())
                        }
                    }
                }

                impl<T> Debug for Error<T>
                where
                    Timestamp: TryFrom<T>,
                    <Timestamp as TryFrom<T>>::Error: Debug
                {
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        match self {
                            Self::TimestampError(err) => f.debug_tuple("TimestampError")
                                .field(err)
                                .finish(),
                            Self::QueryError(err) => f.debug_tuple("QueryError")
                                .field(err)
                                .finish()
                        }
                    }
                }

                impl<T> Display for Error<T>
                where
                    Timestamp: TryFrom<T>,
                    <Timestamp as TryFrom<T>>::Error: Display
                {
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        match self {
                            Self::TimestampError(err) => {
                                write!(f, "Unable to convert value to timestamp: {err}")
                            },
                            Self::QueryError(err) => {
                                write!(f, "Unable to convert timestamp to query: {err}")
                            }
                        }
                    }
                }

                impl<T> ::core::error::Error for Error<T>
                where
                    Timestamp: TryFrom<T>,
                    <Timestamp as TryFrom<T>>::Error: ::core::error::Error + 'static
                {
                    fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)> {
                        match self {
                            Self::TimestampError(err) => Some(err),
                            Self::QueryError(err) => Some(err)
                        }
                    }
                }
            }

            impl #impl_generics ::influxdb::InfluxDbWriteable for #ident #ty_generics #where_clause {
                type Error = __influxdb_private::Error<#time_field_ty>;

                fn try_into_query<I: Into<String>>(
                    self,
                    name: I
                ) -> ::core::result::Result<::influxdb::WriteQuery, Self::Error> {
                    let timestamp: ::influxdb::Timestamp = self.#time_field
                        .try_into()
                        .map_err(__influxdb_private::Error::TimestampError)?;
                    let mut query = timestamp.try_into_query(name)
                        .map_err(__influxdb_private::Error::QueryError)?;
                    #(
                        query = #field_assignments;
                    )*
                    Ok(query)
                }
            }
        };
    })
}
