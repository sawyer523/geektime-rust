use process_js::{process_from_js, process_into_js};

mod process_js;

#[proc_macro_derive(IntoJs)]
pub fn into_js_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    process_into_js(input).into()
}

#[proc_macro_derive(FromJs)]
pub fn from_js_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    process_from_js(input).into()
}
