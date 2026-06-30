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
        let mut args = MacroArgs::default();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "feature" => {
                    input.parse::<Token![=]>()?;
                    args.feature = input.parse::<LitStr>()?.value();
                }
                "stub_gen" => {
                    input.parse::<Token![=]>()?;
                    if input.peek(LitBool) {
                        let b: LitBool = input.parse()?;
                        args.stub_gen = if b.value() {
                            StubGenMode::Feature(args.feature.clone())
                        } else {
                            StubGenMode::Disabled
                        };
                    } else {
                        args.stub_gen = StubGenMode::Feature(input.parse::<LitStr>()?.value());
                    }
                }
                "pyclass_args" => {
                    let inner;
                    syn::parenthesized!(inner in input);
                    args.pyclass_args = Some(inner.parse::<TokenStream>()?);
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
        Ok(args)
    }
}
