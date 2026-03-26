//! # py-compat
//!
//! Attribute macros for writing Rust types **once** and exposing them to Python
//! via [PyO3] without duplicating your code.
//!
//! ## The problem
//!
//! When you have Rust types that are used natively in your own codebase *and*
//! need to be accessible from Python, the naïve approach is to write two
//! parallel definitions: a plain Rust version and a PyO3-annotated copy.
//! Keeping them in sync as the type evolves is error-prone and expensive.
//!
//! ## The solution
//!
//! `py-compat` lets you annotate a **single definition**. The macros emit two
//! cfg-gated versions at compile time:
//!
//! - **Python build** (`--features python`): the full `#[pyclass]` /
//!   `#[pymethods]` version that PyO3 needs.
//! - **Plain build**: the exact same type with all PyO3 annotations stripped,
//!   so it compiles without any PyO3 dependency.
//!
//! ```rust
//! // ONE definition — works as plain Rust and as a Python class.
//! #[py_compat_struct]
//! pub struct Color {
//!     #[pyo3(get, set)]
//!     pub r: u8,
//!     pub g: u8,
//!     pub b: u8,
//! }
//!
//! #[py_compat_methods]
//! impl Color {
//!     // Exposed to both Rust callers and Python.
//!     pub fn to_hex(&self) -> String {
//!         format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
//!     }
//!
//!     // Python-only constructor — not compiled into plain Rust builds.
//!     #[py_only]
//!     #[new]
//!     pub fn py_new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
//! }
//! ```
//!
//! This is equivalent to writing:
//!
//! ```rust
//! #[cfg(feature = "python")]
//! #[pyclass]
//! pub struct Color { #[pyo3(get, set)] pub r: u8, pub g: u8, pub b: u8 }
//!
//! #[cfg(not(feature = "python"))]
//! pub struct Color { pub r: u8, pub g: u8, pub b: u8 }
//! // ... and the same split for every impl block.
//! ```
//!
//! ## Macros
//!
//! | Macro | Applies to |
//! |---|---|
//! | [`py_compat_struct`] | `struct` definitions |
//! | [`py_compat_enum`] | `enum` definitions (simple and complex) |
//! | [`py_compat_methods`] | `impl` blocks |
//!
//! ## Per-item sentinels (used inside `#[py_compat_methods]`)
//!
//! | Attribute | Effect |
//! |---|---|
//! | `#[py_only]` | Item appears only in the Python build (e.g. `#[new]`, `__repr__`) |
//! | `#[py_attrs]` | Item appears in both builds, but all attrs are stripped in the plain build |
//!
//! ## Macro arguments
//!
//! All three macros accept the same optional arguments:
//!
//! | Argument | Values | Default |
//! |---|---|---|
//! | `feature` | `"string"` | `"python"` |
//! | `stub_gen` | `false` / `true` / `"feature-name"` | `"python"` |
//! | `pyclass_args` | token tree forwarded into `#[pyclass(...)]` | _(none)_ |
//!
//! By default the `feature` argument is set to `"python"` while `stub_gen` is
//! disabled (`false`). To enable automatic stub registration set `stub_gen = true`
//! or pass a feature-name (for example `stub_gen = "stub-gen"`).
//!
//! ## Python stub file generation (`.pyi`)
//!
//! When the `python` feature is enabled, each macro automatically emits the
//! appropriate [`pyo3-stub-gen`] derive so that type information is registered
//! at compile time and picked up by the stub-generation binary:
//!
//! | Macro | Emitted stub derive |
//! |---|---|
//! | `py_compat_struct` | `gen_stub_pyclass` |
//! | `py_compat_enum` (unit variants) | `gen_stub_pyclass_enum` |
//! | `py_compat_enum` (struct/tuple variants) | `gen_stub_pyclass_complex_enum` |
//! | `py_compat_methods` | `gen_stub_pymethods` |
//!
//! The correct variant is chosen automatically — you never need to specify it.
//! To finish the setup, call [`define_stub_info_gatherer!`] once in your
//! `lib.rs`. The `stub-gen` feature (which just implies `python`) exists solely
//! to gate the `stub_gen` binary so it isn't compiled into normal builds:
//!
//! ```rust
//! // lib.rs
//! #[cfg(feature = "python")]
//! pyo3_stub_gen::define_stub_info_gatherer!(stub_info);
//! ```
//!
//! To opt a specific type out of stub generation, pass `stub_gen = false`:
//!
//! ```rust
//! #[py_compat_struct(stub_gen = false)]
//! pub struct Internal { ... }
//! ```
//!
//! If your project uses a different feature layout where `pyo3-stub-gen` lives
//! under a separate feature, pass the feature name explicitly:
//!
//! ```rust
//! #[py_compat_struct(stub_gen = "stub-gen")]
//! pub struct Foo { ... }
//! ```
//!
//! ## Cargo setup
//!
//! ```toml
//! [features]
//! default          = []
//! # pyo3 and pyo3-stub-gen are both gated by "python" — one flag enables
//! # PyO3 bindings, stub registration, and all py-compat gating together.
//! python           = ["dep:pyo3", "dep:pyo3-stub-gen"]
//! python-extension = ["python", "pyo3/extension-module", "pyo3/generate-import-lib"]
//! # stub-gen adds no new deps; it only exists to gate the stub_gen binary
//! # so it isn't compiled into library or extension builds.
//! stub-gen         = ["python"]
//!
//! [dependencies]
//! pyo3          = { version = "0.28.0", optional = true }
//! pyo3-stub-gen = { version = "0.6",  optional = true }
//!
//! [[bin]]
//! name             = "stub_gen"
//! path             = "src/bin/stub_gen.rs"
//! required-features = ["stub-gen"]
//! ```
//!
//! [`pyo3-stub-gen`]: https://github.com/Jij-Inc/pyo3-stub-gen
//! [`define_stub_info_gatherer!`]: https://docs.rs/pyo3-stub-gen/latest/pyo3_stub_gen/macro.define_stub_info_gatherer.html
//! [PyO3]: https://pyo3.rs

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, ImplItem, ItemEnum, ItemImpl, ItemStruct, LitBool, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct MacroArgs {
    feature: String,
    stub_gen: Option<String>,
    pyclass_args: Option<TokenStream2>,
}

impl Default for MacroArgs {
    fn default() -> Self {
        Self {
            feature: "python".to_string(),
            stub_gen: None,
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
                        args.stub_gen = b.value().then(|| "python".to_string());
                    } else {
                        args.stub_gen = Some(input.parse::<LitStr>()?.value());
                    }
                }
                "pyclass_args" => {
                    let inner;
                    syn::parenthesized!(inner in input);
                    args.pyclass_args = Some(inner.parse::<TokenStream2>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown argument `{other}`; \
                             expected `feature`, `stub_gen`, or `pyclass_args`"
                        ),
                    ));
                }
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(args)
    }
}

fn is_pyo3_related(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .first()
        .map(|s| {
            matches!(
                s.ident.to_string().as_str(),
                "pyo3" | "pyclass" | "pymethods" | "pyfunction" | "pymodule"
            )
        })
        .unwrap_or(false)
}

fn is_sentinel(attr: &Attribute) -> bool {
    attr.path().is_ident("py_only") || attr.path().is_ident("py_attrs")
}

fn is_gen_stub(attr: &Attribute) -> bool {
    attr.path().is_ident("gen_stub")
}

fn strip_gen_stub_from_item(item: &mut ImplItem) {
    match item {
        ImplItem::Fn(f) => f.attrs.retain(|a| !is_gen_stub(a)),
        ImplItem::Const(c) => c.attrs.retain(|a| !is_gen_stub(a)),
        ImplItem::Type(t) => t.attrs.retain(|a| !is_gen_stub(a)),
        ImplItem::Macro(m) => m.attrs.retain(|a| !is_gen_stub(a)),
        _ => {}
    }
}

fn strip_gen_stub_from_fields(fields: &mut syn::Fields) {
    let iter: Box<dyn Iterator<Item = &mut syn::Field>> = match fields {
        syn::Fields::Named(f) => Box::new(f.named.iter_mut()),
        syn::Fields::Unnamed(f) => Box::new(f.unnamed.iter_mut()),
        syn::Fields::Unit => return,
    };
    for field in iter {
        field.attrs.retain(|a| !is_gen_stub(a));
    }
}

fn strip_gen_stub_from_variants(
    variants: &mut syn::punctuated::Punctuated<syn::Variant, Token![,]>,
) {
    for variant in variants.iter_mut() {
        variant.attrs.retain(|a| !is_gen_stub(a));
        strip_gen_stub_from_fields(&mut variant.fields);
    }
}

fn impl_item_attrs(item: &ImplItem) -> &[Attribute] {
    match item {
        ImplItem::Fn(f) => &f.attrs,
        ImplItem::Const(c) => &c.attrs,
        ImplItem::Type(t) => &t.attrs,
        ImplItem::Macro(m) => &m.attrs,
        _ => &[],
    }
}

fn clear_impl_item_attrs(item: &mut ImplItem) {
    match item {
        ImplItem::Fn(f) => f.attrs.clear(),
        ImplItem::Const(c) => c.attrs.clear(),
        ImplItem::Type(t) => t.attrs.clear(),
        ImplItem::Macro(m) => m.attrs.clear(),
        _ => {}
    }
}

fn strip_sentinels(item: &mut ImplItem) {
    match item {
        ImplItem::Fn(f) => f.attrs.retain(|a| !is_sentinel(a)),
        ImplItem::Const(c) => c.attrs.retain(|a| !is_sentinel(a)),
        ImplItem::Type(t) => t.attrs.retain(|a| !is_sentinel(a)),
        ImplItem::Macro(m) => m.attrs.retain(|a| !is_sentinel(a)),
        _ => {}
    }
}

fn strip_pyo3_from_fields(fields: &mut syn::Fields) {
    let iter: Box<dyn Iterator<Item = &mut syn::Field>> = match fields {
        syn::Fields::Named(f) => Box::new(f.named.iter_mut()),
        syn::Fields::Unnamed(f) => Box::new(f.unnamed.iter_mut()),
        syn::Fields::Unit => return,
    };
    for field in iter {
        field.attrs.retain(|a| !is_pyo3_related(a));
    }
}

fn strip_pyo3_from_variants(variants: &mut syn::punctuated::Punctuated<syn::Variant, Token![,]>) {
    for variant in variants.iter_mut() {
        variant.attrs.retain(|a| !is_pyo3_related(a));
        strip_pyo3_from_fields(&mut variant.fields);
    }
}

fn is_simple_enum(item: &ItemEnum) -> bool {
    item.variants
        .iter()
        .all(|v| matches!(v.fields, syn::Fields::Unit))
}

enum StubKind {
    Struct,
    SimpleEnum,
    ComplexEnum,
    Methods,
    Function,
}

/// Produces the appropriate `#[cfg_attr(feature = "...", gen_stub_*)]` token
/// stream for each kind, or an empty stream when stub-gen is disabled.
fn stub_attr(stub_gen: &Option<String>, kind: StubKind) -> TokenStream2 {
    let Some(sg) = stub_gen else {
        return quote! {};
    };
    match kind {
        StubKind::Struct => quote! {
            #[cfg_attr(feature = #sg, ::pyo3_stub_gen::derive::gen_stub_pyclass)]
        },
        StubKind::SimpleEnum => quote! {
            #[cfg_attr(feature = #sg, ::pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
        },
        StubKind::ComplexEnum => quote! {
            #[cfg_attr(feature = #sg, ::pyo3_stub_gen::derive::gen_stub_pyclass_complex_enum)]
        },
        StubKind::Methods => quote! {
            #[cfg_attr(feature = #sg, ::pyo3_stub_gen::derive::gen_stub_pymethods)]
        },
        StubKind::Function => quote! {
            #[cfg_attr(feature = #sg, ::pyo3_stub_gen::derive::gen_stub_pyfunction)]
        },
    }
}

/// Expose a struct to Python via PyO3 without duplicating the definition.
///
/// The macro emits two cfg-gated versions of the struct:
/// - **Python build**: annotated with `#[pyclass]`.
/// - **Plain build**: identical struct with all PyO3 attrs stripped — no PyO3
///   dependency required.
///
/// When the `stub-gen` feature is active, `#[gen_stub_pyclass]` is
/// automatically added so the type appears in generated `.pyi` files.
///
/// # Arguments
///
/// - `feature = "name"` — Cargo feature that enables the Python build
///   (default: `"python"`).
/// - `stub_gen = false | true | "feature-name"` — controls the
///   `pyo3-stub-gen` `cfg_attr` (default: `"python"`; `false` disables it).
/// - `pyclass_args(...)` — tokens forwarded verbatim into `#[pyclass(...)]`,
///   e.g. `pyclass_args(get_all, rename_all = "camelCase")`.
///
/// # Example
///
/// ```rust
/// // Plain usage — all defaults apply.
/// #[py_compat_struct]
/// pub struct Point {
///     #[pyo3(get, set)]  // stripped in plain build, kept in Python build
///     pub x: f64,
///     pub y: f64,
/// }
///
/// // Custom feature, stub-gen disabled, extra pyclass args.
/// #[py_compat_struct(feature = "pyo3", stub_gen = false, pyclass_args(get_all))]
/// pub struct Rect { pub w: f64, pub h: f64 }
/// ```
#[proc_macro_attribute]
pub fn py_compat_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MacroArgs);
    let feature = &args.feature;

    let input_struct = parse_macro_input!(input as ItemStruct);

    let mut py_struct = input_struct.clone();
    let mut plain_struct = input_struct;

    plain_struct
        .attrs
        .retain(|a| !is_pyo3_related(a) && !is_gen_stub(a));
    strip_pyo3_from_fields(&mut plain_struct.fields);
    strip_gen_stub_from_fields(&mut plain_struct.fields);

    if args.stub_gen.is_none() {
        py_struct.attrs.retain(|a| !is_gen_stub(a));
        strip_gen_stub_from_fields(&mut py_struct.fields);
    }

    let stub = stub_attr(&args.stub_gen, StubKind::Struct);
    let pyclass_inner = args
        .pyclass_args
        .as_ref()
        .map_or(quote! {}, |a| quote! { (#a) });

    quote! {
        #stub
        #[cfg(feature = #feature)]
        #[::pyo3::pyclass #pyclass_inner]
        #py_struct

        #[cfg(not(feature = #feature))]
        #plain_struct
    }
    .into()
}

/// Expose an enum to Python via PyO3 without duplicating the definition.
///
/// Supports both **simple enums** (unit variants only) and **complex enums**
/// (struct or tuple variants). PyO3 generates a class attribute per variant
/// for simple enums, and a constructor per variant for complex ones.
///
/// The macro emits two cfg-gated versions:
/// - **Python build**: annotated with `#[pyclass]`.
/// - **Plain build**: all PyO3 attrs stripped from the enum, every variant,
///   and every variant's fields.
///
/// When the `stub-gen` feature is active, the correct stub derive is chosen
/// automatically based on variant shape:
/// - `gen_stub_pyclass_enum` for unit-only enums
/// - `gen_stub_pyclass_complex_enum` for struct/tuple variants
///
/// Accepts the same arguments as [`py_compat_struct`].
///
/// # Arguments
///
/// - `feature = "name"` — Cargo feature that enables the Python build
///   (default: `"python"`).
/// - `stub_gen = false | true | "feature-name"` — controls the
///   `pyo3-stub-gen` `cfg_attr` (default: `"python"`; `false` disables it).
/// - `pyclass_args(...)` — tokens forwarded verbatim into `#[pyclass(...)]`,
///   e.g. `pyclass_args(eq)`.
///
/// # Example
///
/// ```rust
/// // Simple enum — variants become Python class attributes.
/// // Stub: gen_stub_pyclass_enum
/// #[py_compat_enum]
/// #[derive(Clone, PartialEq)]
/// pub enum Direction { North, South, East, West }
///
/// // Complex enum — each variant becomes its own Python class.
/// // Stub: gen_stub_pyclass_complex_enum
/// #[py_compat_enum(pyclass_args(eq))]
/// pub enum Shape {
///     Circle { radius: f64 },
///     Rect { w: f64, h: f64 },
/// }
///
/// // Variant-level pyo3 attrs are stripped in plain builds.
/// #[py_compat_enum]
/// pub enum Status {
///     #[pyo3(name = "OK")]
///     Ok,
///     #[pyo3(name = "ERR")]
///     Err,
/// }
/// ```
#[proc_macro_attribute]
pub fn py_compat_enum(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MacroArgs);
    let feature = &args.feature;

    let input_enum = parse_macro_input!(input as ItemEnum);

    let stub_kind = if is_simple_enum(&input_enum) {
        StubKind::SimpleEnum
    } else {
        StubKind::ComplexEnum
    };

    let mut py_enum = input_enum.clone();
    let mut plain_enum = input_enum;

    plain_enum
        .attrs
        .retain(|a| !is_pyo3_related(a) && !is_gen_stub(a));
    strip_pyo3_from_variants(&mut plain_enum.variants);
    strip_gen_stub_from_variants(&mut plain_enum.variants);

    if args.stub_gen.is_none() {
        py_enum.attrs.retain(|a| !is_gen_stub(a));
        strip_gen_stub_from_variants(&mut py_enum.variants);
    }

    let stub = stub_attr(&args.stub_gen, stub_kind);
    let pyclass_inner = args
        .pyclass_args
        .as_ref()
        .map_or(quote! {}, |a| quote! { (#a) });

    quote! {
        #stub
        #[cfg(feature = #feature)]
        #[::pyo3::pyclass #pyclass_inner]
        #py_enum

        #[cfg(not(feature = #feature))]
        #plain_enum
    }
    .into()
}

/// Expose an `impl` block to Python via PyO3 without duplicating it.
///
/// Emits two cfg-gated impl blocks from a single source:
/// - **Python build**: annotated with `#[pymethods]`.
/// - **Plain build**: all PyO3 attrs stripped, Python-only items removed.
///
/// When the `stub-gen` feature is active, `#[gen_stub_pymethods]` is
/// automatically added so all methods appear in generated `.pyi` files.
///
/// ## Per-item sentinels
///
/// Add these to individual items inside the block to control which build they
/// appear in. Both are stripped from the final output — they are never emitted.
///
/// - **`#[py_only]`** — item appears only in the Python build. Use this for
///   `#[new]`, `__repr__`, `__str__`, and other Python protocol methods that
///   have no meaning on the Rust side.
///
/// - **`#[py_attrs]`** — item appears in both builds, but *all* attributes are
///   stripped in the plain build. Use this for methods that carry `#[getter]`
///   or `#[setter]` in the Python build but should still be callable from Rust.
///
/// Using both sentinels on the same item is a compile error.
///
/// Accepts the same arguments as [`py_compat_struct`].
///
/// # Arguments
///
/// - `feature = "name"` — Cargo feature that enables the Python build
///   (default: `"python"`).
/// - `stub_gen = false | true | "feature-name"` — controls the
///   `pyo3-stub-gen` `cfg_attr` (default: `"python"`; `false` disables it).
/// - `pyclass_args(...)` — tokens forwarded verbatim into `#[pyclass(...)]`,
///   e.g. `pyclass_args(get_all)`.
///
/// # Example
///
/// ```rust
/// #[py_compat_methods]
/// impl Color {
///     // Compiled into both builds unchanged.
///     pub fn to_hex(&self) -> String {
///         format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
///     }
///
///     // Python-only: Rust callers use the struct literal instead.
///     #[py_only]
///     #[new]
///     pub fn py_new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
///
///     // `#[getter]` kept in Python build; plain build gets a clean `fn r()`.
///     #[py_attrs]
///     #[getter]
///     pub fn r(&self) -> u8 { self.r }
/// }
/// ```
#[proc_macro_attribute]
pub fn py_compat_methods(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MacroArgs);
    let feature = &args.feature;

    let input_impl = parse_macro_input!(input as ItemImpl);
    let self_ty = &input_impl.self_ty;
    let (impl_generics, ty_generics, where_clause) = input_impl.generics.split_for_impl();

    let pass_through_attrs: Vec<_> = input_impl
        .attrs
        .iter()
        .filter(|a| !is_pyo3_related(a))
        .collect();

    let stub_gen_disabled = args.stub_gen.is_none();

    let mut py_items = Vec::<TokenStream2>::new();
    let mut plain_items = Vec::<TokenStream2>::new();

    for item in &input_impl.items {
        let attrs = impl_item_attrs(item);
        let is_py_only = attrs.iter().any(|a| a.path().is_ident("py_only"));
        let is_py_attrs = attrs.iter().any(|a| a.path().is_ident("py_attrs"));

        if is_py_only && is_py_attrs {
            return syn::Error::new_spanned(
                quote! { #item },
                "`#[py_only]` and `#[py_attrs]` cannot both appear on the same item",
            )
            .to_compile_error()
            .into();
        }

        let mut clean = item.clone();
        strip_sentinels(&mut clean);

        if is_py_only {
            if stub_gen_disabled {
                strip_gen_stub_from_item(&mut clean);
            }

            py_items.push(quote! { #clean });
        } else if is_py_attrs {
            let mut stripped = clean.clone();
            clear_impl_item_attrs(&mut stripped);

            if stub_gen_disabled {
                strip_gen_stub_from_item(&mut clean);
            }

            py_items.push(quote! { #clean });
            plain_items.push(quote! { #stripped });
        } else {
            // Plain build: ALWAYS strip gen_stub — pyo3-stub-gen never active.
            // Python build: strip gen_stub only when stub_gen is disabled.
            let mut py_clean = clean.clone();
            let mut plain_clean = clean;

            if stub_gen_disabled {
                strip_gen_stub_from_item(&mut py_clean);
            }
            strip_gen_stub_from_item(&mut plain_clean);

            py_items.push(quote! { #py_clean });
            plain_items.push(quote! { #plain_clean });
        }
    }

    let stub = stub_attr(&args.stub_gen, StubKind::Methods);

    quote! {
        #stub
        #[cfg(feature = #feature)]
        #[::pyo3::pymethods]
        #(#pass_through_attrs)*
        impl #impl_generics #self_ty #ty_generics #where_clause {
            #(#py_items)*
        }

        #[cfg(not(feature = #feature))]
        #(#pass_through_attrs)*
        impl #impl_generics #self_ty #ty_generics #where_clause {
            #(#plain_items)*
        }
    }
    .into()
}

/// Expose a free function to Python via PyO3 without duplicating the definition.
///
/// Emits two cfg-gated versions of the function:
/// - **Python build**: annotated with `#[pyfunction]`.
/// - **Plain build**: identical function with all PyO3 attrs stripped — callable
///   from Rust with no PyO3 dependency.
///
/// When the `python` feature is active, `#[gen_stub_pyfunction]` is
/// automatically added so the function appears in generated `.pyi` files.
///
/// Note: the function's return type must satisfy `Into<PyErr>` for the error
/// case when building with the `python` feature. See the error handling section
/// below.
///
/// Accepts the same arguments as [`py_compat_struct`].
///
/// # Example
///
/// ```rust
/// #[py_compat_fn]
/// pub fn add(a: u32, b: u32) -> u32 {
///     a + b
/// }
///
/// // With pyo3 signature annotation (kept in Python build, stripped in plain):
/// #[py_compat_fn]
/// #[pyo3(signature = (a, b=0))]
/// pub fn add_with_default(a: u32, b: u32) -> u32 {
///     a + b
/// }
/// ```
#[proc_macro_attribute]
pub fn py_compat_fn(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MacroArgs);
    let feature = &args.feature;

    let input_fn = parse_macro_input!(input as syn::ItemFn);
    let py_fn = input_fn.clone();

    // Plain build: strip all pyo3-related attrs (e.g. #[pyo3(signature = ...)]).
    let mut plain_fn = input_fn;
    plain_fn.attrs.retain(|a| !is_pyo3_related(a));

    let stub = stub_attr(&args.stub_gen, StubKind::Function);

    quote! {
        #stub
        #[cfg(feature = #feature)]
        #[::pyo3::pyfunction]
        #py_fn

        #[cfg(not(feature = #feature))]
        #plain_fn
    }
    .into()
}
