use proc_macro_crate::FoundCrate;
use syn::parse_quote;
use uuid::Uuid;

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

#[proc_macro]
pub fn uuid(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::LitStr);

    let ori_vst = find_ori_vst();

    let uuid = match Uuid::parse_str(&input.value()) {
        Ok(uuid) => uuid,
        Err(err) => {
            return syn::Error::new_spanned(input, err)
                .to_compile_error()
                .into();
        }
    };

    let uuid = uuid.as_u128();

    let output = quote::quote! {
        #ori_vst::Uuid::from_u128(#uuid)
    };

    output.into()
}
