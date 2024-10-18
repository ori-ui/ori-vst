use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, Token};

use crate::find_ori_vst;

syn::custom_keyword!(group);
syn::custom_keyword!(name);
syn::custom_keyword!(short);
syn::custom_keyword!(unit);
syn::custom_keyword!(steps);

#[derive(Default)]
struct Attributes {
    group: bool,
    name: Option<String>,
    short: Option<String>,
    unit: Option<syn::Expr>,
    steps: Option<syn::Expr>,
}

impl Attributes {
    fn new(attrs: &[syn::Attribute]) -> Result<Self, syn::Error> {
        let mut group = false;
        let mut name = None;
        let mut short = None;
        let mut unit = None;
        let mut steps = None;

        for attr in attrs {
            if attr.path().is_ident("param") {
                attr.parse_args_with(|input: ParseStream| {
                    loop {
                        if input.is_empty() {
                            break;
                        }

                        if input.parse::<group>().is_ok() {
                            if group {
                                return Err(syn::Error::new_spanned(
                                    group,
                                    "duplicate group attribute",
                                ));
                            }

                            group = true;
                        } else if input.parse::<name>().is_ok() {
                            input.parse::<Token![=]>()?;
                            let name_value = input.parse::<syn::LitStr>()?;

                            if name.is_some() {
                                return Err(syn::Error::new_spanned(
                                    name_value,
                                    "duplicate name attribute",
                                ));
                            }

                            if name_value.value().is_empty() {
                                return Err(syn::Error::new_spanned(
                                    name_value,
                                    "name value cannot be empty",
                                ));
                            }

                            name = Some(name_value.value());
                        } else if input.parse::<short>().is_ok() {
                            input.parse::<Token![=]>()?;
                            let short_value = input.parse::<syn::LitStr>()?;

                            if short.is_some() {
                                return Err(syn::Error::new_spanned(
                                    short_value,
                                    "duplicate short attribute",
                                ));
                            }

                            if short_value.value().is_empty() {
                                return Err(syn::Error::new_spanned(
                                    short_value,
                                    "short value cannot be empty",
                                ));
                            }

                            short = Some(short_value.value());
                        } else if input.parse::<unit>().is_ok() {
                            input.parse::<Token![=]>()?;
                            let unit_value = input.parse::<syn::Expr>()?;

                            if unit.is_some() {
                                return Err(syn::Error::new_spanned(
                                    unit_value,
                                    "duplicate unit attribute",
                                ));
                            }

                            unit = Some(unit_value);
                        } else if input.parse::<steps>().is_ok() {
                            input.parse::<Token![=]>()?;
                            let steps_value = input.parse::<syn::Expr>()?;

                            if steps.is_some() {
                                return Err(syn::Error::new_spanned(
                                    steps_value,
                                    "duplicate steps attribute",
                                ));
                            }

                            steps = Some(steps_value);
                        } else {
                            return Err(input.error("expected `group`"));
                        }

                        if input.is_empty() {
                            break;
                        }

                        input.parse::<Token![,]>()?;
                    }

                    Ok(())
                })?;
            }
        }

        let is_param = name.is_some() || short.is_some() || unit.is_some() || steps.is_some();
        if group && is_param {
            return Err(syn::Error::new_spanned(
                name,
                "group attribute cannot be used with name, short, or unit attributes",
            ));
        }

        Ok(Self {
            group,
            name,
            short,
            unit,
            steps,
        })
    }
}

pub fn derive_params(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let ori_vst = find_ori_vst();

    let ident = &input.ident;
    let fields = get_fields(&input);

    let count = count(&fields);
    let info = info(&fields);
    let param = param(&fields);
    let identifier = identifier(&fields);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #ori_vst::Params for #ident #ty_generics #where_clause {
            fn count(&self) -> ::std::primitive::usize {
                #count
            }

            fn info(
                &self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<#ori_vst::ParamInfo> {
                #info
            }

            fn param(
                &mut self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&mut dyn #ori_vst::Param> {
                #param
            }

            fn identifier(
                &self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<::std::string::String> {
                #identifier
            }
        }
    };

    expanded.into()
}

fn count(fields: &[syn::Field]) -> TokenStream {
    let ori_vst = find_ori_vst();

    let fields = fields
        .iter()
        .map(|field| -> Result<TokenStream, syn::Error> {
            let ident = &field.ident;
            let attrs = Attributes::new(&field.attrs)?;

            if attrs.group {
                Ok(quote! {
                    #ori_vst::Params::count(&self.#ident)
                })
            } else {
                Ok(quote!(1))
            }
        })
        .map(|result| match result {
            Ok(value) => value,
            Err(err) => err.to_compile_error(),
        });

    quote! {
        0 #(+ #fields)*
    }
}

fn info(fields: &[syn::Field]) -> TokenStream {
    let ori_vst = find_ori_vst();

    let fields = fields
        .iter()
        .map(|field| -> Result<TokenStream, syn::Error> {
            let ident = field.ident.as_ref().unwrap();
            let attrs = Attributes::new(&field.attrs)?;

            if attrs.group {
                Ok(quote! {
                    match #ori_vst::Params::info(&self.#ident, index - __count) {
                        ::std::option::Option::Some(info) => {
                            return ::std::option::Option::Some(info);
                        }
                        _ => {
                            __count += #ori_vst::Params::count(&self.#ident);
                        }
                    }
                })
            } else {
                let name = attrs.name.clone().unwrap_or_else(|| ident.to_string());
                let short = attrs.short.clone().unwrap_or_else(|| ident.to_string());

                let unit = match &attrs.unit {
                    Some(unit) => quote! { #unit },
                    None => quote! { #ori_vst::Param::unit(&self.#ident) },
                };

                let steps = match &attrs.steps {
                    Some(steps) => quote! { #steps },
                    None => quote! { #ori_vst::Param::steps(&self.#ident).unwrap_or(0) },
                };

                Ok(quote! {
                    if index == __count {
                        return ::std::option::Option::Some(#ori_vst::ParamInfo {
                            name: ::std::string::String::from(#name),
                            short: ::std::string::String::from(#short),
                            unit: #unit,
                            step_count: #steps,
                            default_normalized: #ori_vst::Param::default_normalized(&self.#ident),
                            flags: #ori_vst::Param::flags(&self.#ident),
                        });
                    } else {
                        __count += 1;
                    }
                })
            }
        })
        .map(|result| match result {
            Ok(value) => value,
            Err(err) => err.to_compile_error(),
        });

    quote! {
        let mut __count = 0;

        #(#fields)*

        ::std::option::Option::None
    }
}

fn param(fields: &[syn::Field]) -> TokenStream {
    let ori_vst = find_ori_vst();

    let fields = fields
        .iter()
        .map(|field| -> Result<TokenStream, syn::Error> {
            let ident = &field.ident;
            let attrs = Attributes::new(&field.attrs)?;

            if attrs.group {
                Ok(quote! {
                    if index < __count + #ori_vst::Params::count(&self.#ident) {
                        return #ori_vst::Params::param(&mut self.#ident, index - __count);
                    } else {
                        __count += #ori_vst::Params::count(&self.#ident);
                    }
                })
            } else {
                Ok(quote! {
                    if index == __count {
                        return ::std::option::Option::Some(&mut self.#ident);
                    } else {
                        __count += 1;
                    }
                })
            }
        })
        .map(|result| match result {
            Ok(value) => value,
            Err(err) => err.to_compile_error(),
        });

    quote! {
        let mut __count = 0;

        #(#fields)*

        ::std::option::Option::None
    }
}

fn identifier(fields: &[syn::Field]) -> TokenStream {
    let ori_vst = find_ori_vst();

    let fields = fields
        .iter()
        .map(|field| -> Result<TokenStream, syn::Error> {
            let ident = &field.ident;
            let attrs = Attributes::new(&field.attrs)?;

            if attrs.group {
                Ok(quote! {
                    if index < __count + #ori_vst::Params::count(&self.#ident) {
                        return ::std::option::Option::Some(
                            ::std::format!(
                                "{}_{}",
                                ::std::stringify!(#ident),
                                #ori_vst::Params::identifier(&self.#ident, index - __count)?,
                            ),
                        );
                    } else {
                        __count += #ori_vst::Params::count(&self.#ident);
                    }
                })
            } else {
                Ok(quote! {
                    if index == __count {
                        return ::std::option::Option::Some(
                            ::std::string::String::from(::std::stringify!(#ident)),
                        );
                    } else {
                        __count += 1;
                    }
                })
            }
        })
        .map(|result| match result {
            Ok(value) => value,
            Err(err) => err.to_compile_error(),
        });

    quote! {
        let mut __count = 0;

        #(#fields)*

        ::std::option::Option::None
    }
}

fn get_fields(input: &syn::DeriveInput) -> Vec<syn::Field> {
    match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields.named.iter().cloned().collect(),
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    }
}
