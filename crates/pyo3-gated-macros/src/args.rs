use proc_macro2::TokenStream;
use syn::{
    LitBool, LitStr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub(crate) struct MacroArgs {
    pub feature: String,
    pub stub_gen: StubGenMode,
    pub pyclass_args: Option<TokenStream>,
    pub pyfunction_args: Option<TokenStream>,
    pub pyo3_crate: Option<TokenStream>,
    pub pyo3_crate_attr: Option<String>,
    pub py_only: bool,
}

pub(crate) struct ModuleArgs {
    pub feature: String,
    pub pyo3_crate: Option<TokenStream>,
    pub pyo3_crate_attr: Option<String>,
    pub module: syn::Ident,
    pub classes: Vec<syn::Type>,
    pub functions: Vec<syn::Path>,
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
            stub_gen: StubGenMode::Feature("stub-gen".to_string()),
            pyclass_args: None,
            pyfunction_args: None,
            pyo3_crate: None,
            pyo3_crate_attr: None,
            py_only: false,
        }
    }
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut feature = None::<String>;
        let mut stub_gen = RawStubGen::Unset;
        let mut pyclass_args = None::<TokenStream>;
        let mut pyfunction_args = None::<TokenStream>;
        let mut pyo3_crate = None::<TokenStream>;
        let mut pyo3_crate_attr = None::<String>;
        let mut py_only = false;

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
                "pyfunction_args" => {
                    if pyfunction_args.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate `pyfunction_args` argument",
                        ));
                    }
                    let inner;
                    syn::parenthesized!(inner in input);
                    pyfunction_args = Some(inner.parse::<TokenStream>()?);
                }
                "pyo3_crate" => {
                    if pyo3_crate.is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate `pyo3_crate` argument",
                        ));
                    }
                    input.parse::<Token![=]>()?;
                    let value = input.parse::<LitStr>()?.value();
                    if value.is_empty() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "`pyo3_crate` must not be empty",
                        ));
                    }
                    let path: syn::Path = syn::parse_str(&value).map_err(|_| {
                        syn::Error::new(ident.span(), "`pyo3_crate` must be a valid Rust path")
                    })?;
                    pyo3_crate = Some(quote::quote! { #path });
                    pyo3_crate_attr = Some(value);
                }
                "py_only" => {
                    if py_only {
                        return Err(syn::Error::new(
                            ident.span(),
                            "duplicate `py_only` argument",
                        ));
                    }
                    py_only = true;
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown argument `{other}`; expected `feature`, `stub_gen`, `pyclass_args`, `pyfunction_args`, `pyo3_crate`, or `py_only`"
                        ),
                    ));
                }
            }
            let _ = input.parse::<Token![,]>();
        }

        let feature = feature.unwrap_or_else(|| "python".to_string());
        let stub_gen = match stub_gen {
            RawStubGen::Unset => StubGenMode::Feature("stub-gen".to_string()),
            RawStubGen::SameAsFeature => StubGenMode::Feature(feature.clone()),
            RawStubGen::Feature(feature) => StubGenMode::Feature(feature),
            RawStubGen::Disabled => StubGenMode::Disabled,
        };

        Ok(MacroArgs {
            feature,
            stub_gen,
            pyclass_args,
            pyfunction_args,
            pyo3_crate,
            pyo3_crate_attr,
            py_only,
        })
    }
}

impl MacroArgs {
    pub(crate) fn pyo3_path(&self) -> TokenStream {
        self.pyo3_crate
            .clone()
            .unwrap_or_else(crate::paths::pyo3_crate_path)
    }

    pub(crate) fn pyo3_crate_attr(&self) -> Option<LitStr> {
        crate_attr_literal(self.pyo3_crate_attr.as_deref())
    }

    pub(crate) fn should_strip_stub_gen(&self) -> bool {
        matches!(self.stub_gen, StubGenMode::Disabled)
    }

    pub(crate) fn reject_fn_only_args(&self) -> Option<syn::Error> {
        if self.pyfunction_args.is_some() {
            return Some(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`pyfunction_args` applies only to `#[py_compat_fn]` and function items passed to `#[py_compat]`",
            ));
        }
        if self.py_only {
            return Some(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`py_only` applies only to `#[py_compat_fn]` and function items passed to `#[py_compat]`",
            ));
        }
        None
    }

    pub(crate) fn reject_class_only_args_on_fn(&self) -> Option<syn::Error> {
        if self.pyclass_args.is_some() {
            return Some(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`pyclass_args` applies only to structs/enums; use `pyfunction_args` for functions",
            ));
        }
        None
    }
}

impl ModuleArgs {
    pub(crate) fn pyo3_path(&self) -> TokenStream {
        self.pyo3_crate
            .clone()
            .unwrap_or_else(crate::paths::pyo3_crate_path)
    }

    pub(crate) fn pyo3_crate_attr(&self) -> Option<LitStr> {
        crate_attr_literal(self.pyo3_crate_attr.as_deref())
    }
}

fn crate_attr_literal(override_path: Option<&str>) -> Option<LitStr> {
    let path = override_path.map(ToOwned::to_owned).or_else(|| {
        crate::paths::resolved_pyo3_crate_name().and_then(|name| {
            if name == "pyo3" {
                None
            } else {
                Some(format!("::{name}"))
            }
        })
    })?;
    Some(LitStr::new(&path, proc_macro2::Span::call_site()))
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut feature = "python".to_string();
        let mut pyo3_crate = None::<TokenStream>;
        let mut pyo3_crate_attr = None::<String>;
        let mut module = None::<syn::Ident>;
        let mut classes = Vec::<syn::Type>::new();
        let mut functions = Vec::<syn::Path>::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "feature" => {
                    input.parse::<Token![=]>()?;
                    let value = input.parse::<LitStr>()?.value();
                    if value.is_empty() {
                        return Err(syn::Error::new(ident.span(), "`feature` must not be empty"));
                    }
                    feature = value;
                }
                "pyo3_crate" => {
                    input.parse::<Token![=]>()?;
                    let value = input.parse::<LitStr>()?.value();
                    if value.is_empty() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "`pyo3_crate` must not be empty",
                        ));
                    }
                    let path: syn::Path = syn::parse_str(&value).map_err(|_| {
                        syn::Error::new(ident.span(), "`pyo3_crate` must be a valid Rust path")
                    })?;
                    pyo3_crate = Some(quote::quote! { #path });
                    pyo3_crate_attr = Some(value);
                }
                "module" => {
                    if module.is_some() {
                        return Err(syn::Error::new(ident.span(), "duplicate `module` entry"));
                    }
                    module = Some(input.parse()?);
                }
                "classes" => {
                    input.parse::<Token![:]>()?;
                    let content;
                    syn::bracketed!(content in input);
                    classes = Punctuated::<syn::Type, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "functions" => {
                    input.parse::<Token![:]>()?;
                    let content;
                    syn::bracketed!(content in input);
                    functions = Punctuated::<syn::Path, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown module entry `{other}`; expected `module`, `classes`, `functions`, `feature`, or `pyo3_crate`"
                        ),
                    ));
                }
            }

            let _ = input.parse::<Token![;]>();
            let _ = input.parse::<Token![,]>();
        }

        let module = module.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "`define_py_module!` requires `module <name>;`",
            )
        })?;

        Ok(Self {
            feature,
            pyo3_crate,
            pyo3_crate_attr,
            module,
            classes,
            functions,
        })
    }
}
