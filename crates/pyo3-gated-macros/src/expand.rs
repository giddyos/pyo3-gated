use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemEnum, ItemImpl, ItemStruct};

use crate::args::{MacroArgs, StubGenMode};
use crate::attrs::{
    StubKind, impl_item_attrs, is_gen_stub, is_pyo3_related, is_simple_enum,
    strip_gen_stub_from_fields, strip_gen_stub_from_item, strip_gen_stub_from_variants,
    strip_pyo3_from_fields, strip_pyo3_from_signature, strip_pyo3_from_variants,
    strip_python_attrs_from_impl_item, strip_sentinels, stub_attr,
};

pub(crate) fn expand_struct(args: MacroArgs, input_struct: ItemStruct) -> TokenStream {
    let feature = &args.feature;
    let mut py_struct = input_struct.clone();
    let mut plain_struct = input_struct;

    plain_struct
        .attrs
        .retain(|a| !is_pyo3_related(a) && !is_gen_stub(a));
    strip_pyo3_from_fields(&mut plain_struct.fields);
    strip_gen_stub_from_fields(&mut plain_struct.fields);

    if matches!(args.stub_gen, StubGenMode::Disabled) {
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
        #[allow(unexpected_cfgs)]
        #[cfg(feature = #feature)]
        #[::pyo3::pyclass #pyclass_inner]
        #py_struct

        #[allow(unexpected_cfgs)]
        #[cfg(not(feature = #feature))]
        #plain_struct
    }
}

pub(crate) fn expand_enum(args: MacroArgs, input_enum: ItemEnum) -> TokenStream {
    let feature = &args.feature;
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

    if matches!(args.stub_gen, StubGenMode::Disabled) {
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
        #[allow(unexpected_cfgs)]
        #[cfg(feature = #feature)]
        #[::pyo3::pyclass #pyclass_inner]
        #py_enum

        #[allow(unexpected_cfgs)]
        #[cfg(not(feature = #feature))]
        #plain_enum
    }
}

pub(crate) fn expand_methods(args: MacroArgs, input_impl: ItemImpl) -> TokenStream {
    if input_impl.trait_.is_some() {
        return syn::Error::new_spanned(
            &input_impl.impl_token,
            "`#[py_compat_methods]` only supports inherent `impl Type { ... }` blocks; trait impls are not supported",
        )
        .to_compile_error();
    }

    let feature = &args.feature;
    let self_ty = &input_impl.self_ty;
    let (impl_generics, ty_generics, where_clause) = input_impl.generics.split_for_impl();

    let pass_through_attrs: Vec<_> = input_impl
        .attrs
        .iter()
        .filter(|a| !is_pyo3_related(a))
        .collect();

    let stub_gen_disabled = matches!(args.stub_gen, StubGenMode::Disabled);
    let mut py_items = Vec::<TokenStream>::new();
    let mut plain_items = Vec::<TokenStream>::new();

    for item in &input_impl.items {
        let attrs = impl_item_attrs(item);
        let is_py_only = attrs.iter().any(|a| a.path().is_ident("py_only"));
        let is_py_attrs = attrs.iter().any(|a| a.path().is_ident("py_attrs"));

        if is_py_only && is_py_attrs {
            return syn::Error::new_spanned(
                item,
                "`#[py_only]` and `#[py_attrs]` cannot both appear on the same item",
            )
            .to_compile_error();
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
            strip_python_attrs_from_impl_item(&mut stripped);

            if stub_gen_disabled {
                strip_gen_stub_from_item(&mut clean);
            }

            py_items.push(quote! { #clean });
            plain_items.push(quote! { #stripped });
        } else {
            let mut py_clean = clean.clone();
            let mut plain_clean = clean;

            if stub_gen_disabled {
                strip_gen_stub_from_item(&mut py_clean);
            }
            strip_python_attrs_from_impl_item(&mut plain_clean);

            py_items.push(quote! { #py_clean });
            plain_items.push(quote! { #plain_clean });
        }
    }

    let stub = stub_attr(&args.stub_gen, StubKind::Methods);

    quote! {
        #stub
        #[allow(unexpected_cfgs)]
        #[cfg(feature = #feature)]
        #[::pyo3::pymethods]
        #(#pass_through_attrs)*
        impl #impl_generics #self_ty #ty_generics #where_clause {
            #(#py_items)*
        }

        #[allow(unexpected_cfgs)]
        #[cfg(not(feature = #feature))]
        #(#pass_through_attrs)*
        impl #impl_generics #self_ty #ty_generics #where_clause {
            #(#plain_items)*
        }
    }
}

pub(crate) fn expand_fn(args: MacroArgs, input_fn: syn::ItemFn) -> TokenStream {
    let feature = &args.feature;
    let mut py_fn = input_fn.clone();
    let mut plain_fn = input_fn;
    plain_fn
        .attrs
        .retain(|a| !is_pyo3_related(a) && !is_gen_stub(a));
    strip_pyo3_from_signature(&mut plain_fn.sig);

    if matches!(args.stub_gen, StubGenMode::Disabled) {
        py_fn.attrs.retain(|a| !is_gen_stub(a));
    }

    let stub = stub_attr(&args.stub_gen, StubKind::Function);

    quote! {
        #stub
        #[allow(unexpected_cfgs)]
        #[cfg(feature = #feature)]
        #[::pyo3::pyfunction]
        #py_fn

        #[allow(unexpected_cfgs)]
        #[cfg(not(feature = #feature))]
        #plain_fn
    }
}
