use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	ItemStruct,
	parse_macro_input
};

pub fn expand_writeable(tokens : TokenStream) -> TokenStream
{
	let input = parse_macro_input!(tokens as ItemStruct);
	let ident = input.ident;
	let generics = input.generics;
	
	let output = quote! {
		impl #generics ::influxdb::query::InfluxDbWriteable for #ident #generics
		{
			fn into_query(self) -> InfluxDbWriteQuery
			{
				unimplemented!()
			}
		}
	};
	output.into()
}
