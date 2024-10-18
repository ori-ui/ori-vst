use proc_macro_crate::FoundCrate;
use syn::parse_quote;

mod params;

fn find_ori_vst() -> syn::Path {
    match proc_macro_crate::crate_name("ori_vst") {
        Ok(FoundCrate::Itself) => parse_quote!(crate),
        Ok(FoundCrate::Name(name)) => syn::parse_str(&name).unwrap(),
        Err(_) => syn::parse_str("ori_vst").unwrap(),
    }
}

#[proc_macro_derive(Params, attributes(param))]
pub fn derive_params(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    params::derive_params(input)
}
