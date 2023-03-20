use proc_macro::TokenStream;

mod writeable;
use syn::parse_macro_input;
use writeable::expand_writeable;

#[proc_macro_derive(InfluxDbWriteable, attributes(influxdb))]
pub fn derive_writeable(input: TokenStream) -> TokenStream {
    expand_writeable(parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
