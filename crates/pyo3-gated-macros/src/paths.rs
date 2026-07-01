use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub(crate) fn facade_crate_ident() -> Ident {
    match crate_name("pyo3-gated") {
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()),
        Ok(FoundCrate::Itself) | Err(_) => Ident::new("pyo3_gated", Span::call_site()),
    }
}

pub(crate) fn facade_crate_path() -> TokenStream {
    match crate_name("pyo3-gated") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::pyo3_gated),
    }
}

pub(crate) fn pyo3_stub_gen_path() -> TokenStream {
    let facade = facade_crate_path();
    quote!(#facade::__private::pyo3_stub_gen)
}

pub(crate) fn pyo3_crate_path() -> TokenStream {
    match crate_name("pyo3") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::pyo3),
    }
}

pub(crate) fn pyo3_missing_diagnostic(
    feature: &str,
    override_present: bool,
) -> Option<TokenStream> {
    if override_present || crate_name("pyo3").is_ok() {
        return None;
    }

    Some(quote! {
        #[allow(unexpected_cfgs)]
        #[cfg(feature = #feature)]
        compile_error!("pyo3-gated: enabling the Python feature requires a direct optional `pyo3` dependency, e.g. `pyo3 = { version = \"0.28\", optional = true }`, or a `pyo3_crate = \"...\"` override.");
    })
}

pub(crate) fn resolved_pyo3_crate_name() -> Option<String> {
    match crate_name("pyo3") {
        Ok(FoundCrate::Itself) => Some("pyo3".to_string()),
        Ok(FoundCrate::Name(name)) => Some(name),
        Err(_) => None,
    }
}
