use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod writeable;
use writeable::expand_writeable;

fn krate() -> TokenStream2 {
    quote!(::influxdb)
}

#[proc_macro_derive(InfluxDbWriteable, attributes(influxdb))]
pub fn derive_writeable(tokens: TokenStream) -> TokenStream {
    expand_writeable(tokens)
}
