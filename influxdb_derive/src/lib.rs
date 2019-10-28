extern crate proc_macro;

use proc_macro::TokenStream;

mod writeable;
use writeable::expand_writeable;

#[proc_macro_derive(InfluxDbWriteable, attributes(tag))]
pub fn derive_writeable(tokens: TokenStream) -> TokenStream {
    expand_writeable(tokens)
}
