use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn builder(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original = proc_macro2::TokenStream::from(item);

    return quote! {
        #original
    }
    .into();
}
