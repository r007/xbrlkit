//! `#[derive(FromXbrl)]` for [xbrlkit](https://docs.rs/xbrlkit).
//!
//! Binds the fields of a struct to XBRL concepts. The binding lives in
//! `#[xbrl(..)]` attributes and nowhere else, so the struct's serde names stay
//! its Rust names — which is what reaches JSON, Arrow and Parquet.
//!
//! Depend on `xbrlkit` and use the derive it re-exports; this crate is not
//! meant to be used on its own, and the code it generates refers to
//! `::xbrlkit`.
//!
//! ```ignore
//! use xbrlkit::FromXbrl;
//!
//! #[derive(Default, FromXbrl)]
//! #[xbrl(instant)]
//! pub struct BalanceSheet {
//!     /// The date the figures are as of.
//!     #[xbrl(period_end)]
//!     pub as_of: Option<String>,
//!
//!     #[xbrl(concept = "us-gaap:Assets")]
//!     pub assets: Option<f64>,
//!
//!     /// The first concept the filing reports wins.
//!     #[xbrl(concept = "us-gaap:Liabilities", alias = "us-gaap:LiabilitiesNoncurrent")]
//!     pub liabilities: Option<f64>,
//! }
//! ```
//!
//! ## Field attributes
//!
//! | Attribute                            | The field is                                             |
//! | ------------------------------------ | -------------------------------------------------------- |
//! | `concept = ".."` [, `alias = ".."`]… | read from these concepts, in order of preference         |
//! | `nested`                             | another `FromXbrl` struct, read in the same scope        |
//! | `each_period`                        | a `Vec` of a `FromXbrl` struct, one per reported period  |
//! | `period_start` / `period_end`        | the dates of the period the struct was read for          |
//! | *(none)*                             | left at its `Default`                                    |
//!
//! What a `concept` field holds is decided by its type — see
//! `xbrlkit::bind::FromFacts`.
//!
//! ## Struct attributes
//!
//! `#[xbrl(instant)]` or `#[xbrl(duration)]` says which kind of period the
//! struct's concepts are reported for, so that `each_period` does not produce
//! a balance sheet for a quarter or an income statement for a date.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Field, Fields, LitStr, parse_macro_input, spanned::Spanned};

/// What one field is bound to.
enum Binding {
    Concepts(Vec<LitStr>),
    Nested,
    EachPeriod,
    PeriodStart,
    PeriodEnd,
    Unbound,
}

/// Derives `xbrlkit::FromXbrl` for a struct with named fields. See the crate docs.
#[proc_macro_derive(FromXbrl, attributes(xbrl))]
pub fn derive_from_xbrl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let Data::Struct(data) = &input.data else {
        return Err(Error::new(
            input.span(),
            "FromXbrl can only be derived for a struct",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(Error::new(
            input.span(),
            "FromXbrl needs a struct with named fields",
        ));
    };

    let period_kind = container_period_kind(input)?;

    let mut field_values = Vec::with_capacity(fields.named.len());
    let mut concept_lists = Vec::new();

    for field in &fields.named {
        let ident = field.ident.as_ref().expect("named field");
        let ty = &field.ty;
        let field_name = ident.to_string();

        let value = match field_binding(field)? {
            Binding::Concepts(concepts) => {
                concept_lists.push(quote! { out.extend_from_slice(&[#(#concepts),*]); });
                quote! {
                    <#ty as ::xbrlkit::bind::FromFacts>::from_facts(scope, &[#(#concepts),*], #field_name)
                }
            }
            Binding::Nested => {
                concept_lists.push(quote! { <#ty as ::xbrlkit::bind::FromXbrl>::concepts(out); });
                quote! { <#ty as ::xbrlkit::bind::FromXbrl>::from_xbrl(scope) }
            }
            Binding::EachPeriod => quote! { ::xbrlkit::bind::each_period(scope) },
            Binding::PeriodStart => quote! { scope.period_start() },
            Binding::PeriodEnd => quote! { scope.period_end() },
            Binding::Unbound => quote! { ::core::default::Default::default() },
        };
        field_values.push(quote! { #ident: #value });
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::xbrlkit::bind::FromXbrl for #name #ty_generics #where_clause {
            const PERIOD_KIND: ::xbrlkit::bind::PeriodKind = #period_kind;

            fn concepts(out: &mut ::std::vec::Vec<&'static str>) {
                #(#concept_lists)*
            }

            fn from_xbrl(scope: &::xbrlkit::bind::Scope<'_>) -> Self {
                Self { #(#field_values),* }
            }
        }
    })
}

fn container_period_kind(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let mut kind = quote! { ::xbrlkit::bind::PeriodKind::Any };
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("xbrl")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("instant") {
                kind = quote! { ::xbrlkit::bind::PeriodKind::Instant };
            } else if meta.path.is_ident("duration") {
                kind = quote! { ::xbrlkit::bind::PeriodKind::Duration };
            } else {
                return Err(meta.error("expected `instant` or `duration`"));
            }
            Ok(())
        })?;
    }
    Ok(kind)
}

fn field_binding(field: &Field) -> syn::Result<Binding> {
    let mut binding = Binding::Unbound;
    let mut aliases: Vec<LitStr> = Vec::new();

    for attr in field.attrs.iter().filter(|a| a.path().is_ident("xbrl")) {
        attr.parse_nested_meta(|meta| {
            let path = &meta.path;
            if path.is_ident("alias") {
                aliases.push(meta.value()?.parse()?);
                return Ok(());
            }

            let next = if path.is_ident("concept") {
                Binding::Concepts(vec![meta.value()?.parse()?])
            } else if path.is_ident("nested") {
                Binding::Nested
            } else if path.is_ident("each_period") {
                Binding::EachPeriod
            } else if path.is_ident("period_start") {
                Binding::PeriodStart
            } else if path.is_ident("period_end") {
                Binding::PeriodEnd
            } else {
                return Err(meta.error(
                    "expected `concept = \"..\"`, `alias = \"..\"`, `nested`, `each_period`, \
                     `period_start` or `period_end`",
                ));
            };

            if !matches!(binding, Binding::Unbound) {
                return Err(meta.error("a field can be bound only once"));
            }
            binding = next;
            Ok(())
        })?;
    }

    match &mut binding {
        Binding::Concepts(concepts) => concepts.append(&mut aliases),
        _ if !aliases.is_empty() => {
            return Err(Error::new(
                field.span(),
                "`alias` names a fallback for `concept`, which this field does not have",
            ));
        }
        _ => {}
    }
    Ok(binding)
}
