mod lambda;
mod main;
mod resource;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    main::r#impl(attr, item)
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn lambda(attr: TokenStream, item: TokenStream) -> TokenStream {
    lambda::r#impl(attr, item)
}
