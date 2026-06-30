use proc_macro2::TokenStream;
use syn::{
    LitBool, LitStr, Token,
    parse::{Parse, ParseStream},
};

pub(crate) struct MacroArgs {
    pub feature: String,
    pub stub_gen: StubGenMode,
    pub pyclass_args: Option<TokenStream>,
}

pub(crate) enum StubGenMode {
    Disabled,
    Feature(String),
}

enum RawStubGen {
    Unset,
    Disabled,
    SameAsFeature,
    Feature(String),
}

impl Default for MacroArgs {
    fn default() -> Self {
        Self {
            feature: "python".to_string(),
            stub_gen: StubGenMode::Feature("python".to_string()),
            pyclass_args: None,
        }
    }
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut feature = None::<String>;
        let mut stub_gen = RawStubGen::Unset;
        let mut pyclass_args = None::<TokenStream>;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "feature" => {
                    if feature.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate `feature` argument",
                        ));
                    }
                    input.parse::<Token![=]>()?;
                    let value = input.parse::<LitStr>()?.value();
                    if value.is_empty() {
                        return Err(syn::Error::new(ident.span(), "`feature` must not be empty"));
                    }
                    feature = Some(value);
                }
                "stub_gen" => {
                    if !matches!(stub_gen, RawStubGen::Unset) {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate `stub_gen` argument",
                        ));
                    }
                    input.parse::<Token![=]>()?;
                    if input.peek(LitBool) {
                        let b: LitBool = input.parse()?;
                        stub_gen = if b.value() {
                            RawStubGen::SameAsFeature
                        } else {
                            RawStubGen::Disabled
                        };
                    } else {
                        let value = input.parse::<LitStr>()?.value();
                        if value.is_empty() {
                            return Err(syn::Error::new(
                                ident.span(),
                                "`stub_gen` feature must not be empty",
                            ));
                        }
                        stub_gen = RawStubGen::Feature(value);
                    }
                }
                "pyclass_args" => {
                    if pyclass_args.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate `pyclass_args` argument",
                        ));
                    }
                    let inner;
                    syn::parenthesized!(inner in input);
                    pyclass_args = Some(inner.parse::<TokenStream>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown argument `{other}`; expected `feature`, `stub_gen`, or `pyclass_args`"
                        ),
                    ));
                }
            }
            let _ = input.parse::<Token![,]>();
        }

        let feature = feature.unwrap_or_else(|| "python".to_string());
        let stub_gen = match stub_gen {
            RawStubGen::Unset | RawStubGen::SameAsFeature => StubGenMode::Feature(feature.clone()),
            RawStubGen::Feature(feature) => StubGenMode::Feature(feature),
            RawStubGen::Disabled => StubGenMode::Disabled,
        };

        Ok(MacroArgs {
            feature,
            stub_gen,
            pyclass_args,
        })
    }
}
