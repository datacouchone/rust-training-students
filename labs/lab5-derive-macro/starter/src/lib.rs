//! Lab 5 — derive + attribute proc macros (**starter**).
//!
//! Two techniques are demonstrated in this single proc-macro crate:
//!
//! 1. `#[derive(FieldSummary)]` — a **derive** macro that should generate
//!    `impl FieldSummary for T { fn summary() -> String }`, listing every
//!    field's name and type (in declaration order), skipping any field
//!    marked `#[summary(skip)]`.
//! 2. `#[timed]` — an **attribute** macro that wraps a function body so its
//!    wall-clock execution time is logged via [`std::time::Instant`],
//!    without changing the function's signature or return value. This one
//!    is already fully implemented below — read it for a second example
//!    of `quote!` in action before you tackle the derive.
//!
//! The `FieldSummary` *trait* is intentionally **not** defined in this
//! crate. A crate compiled with `proc-macro = true` may only export
//! macros — ordinary `pub` items (structs, traits, functions) are not
//! usable by downstream crates. Real projects work around this by
//! splitting into a `-macros` crate (this one) plus a small companion
//! runtime crate that owns the trait, then re-exporting both from a
//! facade crate. This lab keeps everything in one crate to stay focused
//! on `syn`/`quote`, so `FieldSummary` the trait is defined directly in
//! each consumer — see `tests/expand.rs` and the crate `README.md`.
//!
//! ## Your job
//!
//! Everything compiles and `cargo test` runs as-is: the derive currently
//! emits a `summary()` that just calls `unimplemented!()`. Find the
//! `// === YOUR CODE: emit the impl block ===` marker below and replace
//! the stub with real code. `README.md` walks through it step by step.

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

// ============================================================================
// Provided: syn AST parsing. Fully written — nothing to change here.
//
// Walks a `DeriveInput`, validates that it describes a struct with named
// fields, and extracts `(ident, type, skip)` for every field, honoring any
// `#[summary(skip)]` attribute. Anything else (enum, union, tuple/unit
// struct) produces a `syn::Error` spanned at the offending token, which
// `.to_compile_error()` turns into a normal rustc-style diagnostic pointing
// straight at the user's bad input — no panics, no `TokenStream::new()`
// empty-error footguns.
// ============================================================================

/// One struct field as seen by the `FieldSummary` derive.
#[allow(dead_code)] // `ident`/`ty`/`skip` go unread until you fill in the YOUR CODE block
struct FieldInfo {
    ident: Ident,
    ty: Type,
    skip: bool,
}

/// Parse a `DeriveInput` into the struct's name plus its field list.
fn parse_fields(input: &DeriveInput) -> syn::Result<(Ident, Vec<FieldInfo>)> {
    let name = input.ident.clone();

    let data_struct = match &input.data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(data_enum) => {
            return Err(syn::Error::new_spanned(
                data_enum.enum_token,
                "FieldSummary can only be derived for structs with named fields, not enums",
            ));
        }
        Data::Union(data_union) => {
            return Err(syn::Error::new_spanned(
                data_union.union_token,
                "FieldSummary can only be derived for structs with named fields, not unions",
            ));
        }
    };

    let named = match &data_struct.fields {
        Fields::Named(named) => named,
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "FieldSummary can only be derived for structs with named fields \
                 (tuple structs and unit structs are not supported)",
            ));
        }
    };

    let mut fields = Vec::with_capacity(named.named.len());
    for field in &named.named {
        let ident = field
            .ident
            .clone()
            .expect("Fields::Named guarantees every field has an ident");
        let ty = field.ty.clone();
        let skip = field_has_skip_attr(field)?;
        fields.push(FieldInfo { ident, ty, skip });
    }

    Ok((name, fields))
}

/// Returns `true` if `field` carries `#[summary(skip)]`.
///
/// Any other `#[summary(...)]` content is a hard parse error, so typos
/// (`#[summary(skp)]`) get caught at compile time instead of being
/// silently ignored.
fn field_has_skip_attr(field: &syn::Field) -> syn::Result<bool> {
    let mut skip = false;
    for attr in &field.attrs {
        if !attr.path().is_ident("summary") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
                Ok(())
            } else {
                Err(meta.error("unsupported `summary` attribute; expected `#[summary(skip)]`"))
            }
        })?;
    }
    Ok(skip)
}

// ============================================================================
// `#[derive(FieldSummary)]`
// ============================================================================

#[proc_macro_derive(FieldSummary, attributes(summary))]
pub fn derive_field_summary(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let (name, fields) = match parse_fields(&input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error().into(),
    };

    // === YOUR CODE: emit the impl block ===
    //
    // Build `impl FieldSummary for #name { fn summary() -> ::std::string::String { .. } }`
    // with `quote!`. `fields` is a `Vec<FieldInfo>`; each entry has:
    //   - `.ident`  (syn::Ident)  — the field's name
    //   - `.ty`     (syn::Type)   — the field's type
    //   - `.skip`   (bool)        — true if `#[summary(skip)]` was present
    //
    // Steps (see README.md for the walkthrough):
    //   1. Filter out fields where `.skip` is true.
    //   2. For each remaining field, produce a `"name: Type"` string.
    //      Hint: `field.ident.to_string()` gives the name; for the type,
    //      `let ty = &field.ty; quote!(#ty).to_string()` gives its source
    //      text. Note: for generic types this inserts spaces around
    //      `<`/`>` (e.g. "Vec < String >" rather than "Vec<String>") —
    //      that's `proc-macro2`'s token-to-string formatting, expected
    //      behavior, not a bug to work around.
    //   3. Join those strings with ", " and return them from `summary()`,
    //      using fully-qualified paths (`::std::format!`, not bare
    //      `format!`) for everything you emit — see the hygiene note
    //      below and README.md's "spans & hygiene" section.
    //
    // Replace the stub below — it currently just panics at runtime so the
    // crate (and its tests) still *compile* while you work.
    //
    // Note the `#field_count` interpolation: `field_count` is computed here,
    // in the macro's own Rust code, and `#field_count` splices its *value*
    // (as a literal) into the generated tokens. Writing a bare `field_count`
    // inside `quote!` (no `#`) would instead emit the literal identifier
    // `field_count` into the generated `summary()` body, where no such
    // variable exists — that's a trap worth remembering once you start
    // writing the real generation logic below.
    //
    // Hygiene: every path here resolves at the derive's *call site*, not
    // in this crate — fully qualify everything you emit, even `String`
    // and standard-library macros like `unimplemented!`, as
    // `::std::string::String` / `::std::unimplemented!` do below. See
    // README.md's "spans & hygiene" section and the reference implementation's
    // crate-level "Hygiene note" doc comment for the full rule. Keep this
    // habit when you write the real `format!`-based generation below the
    // marker.
    let field_count = fields.len();
    let expanded = quote! {
        impl FieldSummary for #name {
            fn summary() -> ::std::string::String {
                ::std::unimplemented!(
                    "TODO(lab5): emit a real summary of the {} parsed field(s), honoring #[summary(skip)]",
                    #field_count
                )
            }
        }
    };
    // === END YOUR CODE ===

    expanded.into()
}

// ============================================================================
// `#[timed]` — provided complete, as a second worked example of `quote!`.
// ============================================================================

/// Wraps a function so its wall-clock execution time is logged to stderr
/// via `std::time::Instant`, without altering its signature or return
/// value.
///
/// The original body is moved into an immediately-invoked closure. Any
/// `return` inside it returns from the closure (not the outer function),
/// so the closure's result *is* whatever the original body would have
/// produced — the outer function then just logs the elapsed time and
/// hands that value back unchanged.
///
/// ## Async functions
///
/// A plain synchronous closure, `(move || #block)()`, breaks on an
/// `async fn`: the body may contain `.await`, but `.await` is only legal
/// directly inside an `async` context, and a bare (non-`async`) closure
/// isn't one — so a wrapped `async fn` fails to compile with a confusing
/// "`.await` is only allowed inside `async` functions and blocks" error
/// pointing at code the user never wrote by hand. We fix this by checking
/// `sig.asyncness`: for an `async fn`, the body is instead moved into an
/// `async move { #block }` block that is immediately `.await`ed —
/// `(async move #block).await` — which both re-establishes a valid async
/// context for the original `.await`s and, because the outer function
/// itself stays `async` (we splice `#sig` verbatim), still returns a
/// `Future` to its own caller rather than blocking. The synchronous path
/// is unchanged.
#[proc_macro_attribute]
pub fn timed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);

    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let fn_name = sig.ident.to_string();

    // `async fn`s need their original body re-wrapped in an `async move`
    // block (then immediately awaited) instead of a synchronous closure —
    // see the "Async functions" section of this fn's doc comment above.
    let invoke_body = if sig.asyncness.is_some() {
        quote! { (async move #block).await }
    } else {
        quote! { (move || #block)() }
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let __lab5_timed_start = ::std::time::Instant::now();
            let __lab5_timed_result = #invoke_body;
            let __lab5_timed_elapsed = __lab5_timed_start.elapsed();
            ::std::eprintln!("[timed] {} took {:?}", #fn_name, __lab5_timed_elapsed);
            __lab5_timed_result
        }
    };

    expanded.into()
}

// ============================================================================
// Unit tests for the provided parsing helper.
//
// These run as ordinary `cargo test` unit tests against this crate's own
// `rlib` build (proc-macro crates are only restricted to macro-only
// exports when consumed as a *dependency* by another crate; their own
// `#[cfg(test)]` code can call private items directly). They demonstrate
// that bad input produces a clear, spanned `syn::Error` rather than a
// panic — see README.md's "spans & hygiene" section. They pass already:
// the parsing helper is provided, not part of your exercise.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// `syn::Type`/`syn::Data` don't implement `Debug` without syn's
    /// `extra-traits` feature (not enabled here), so `Result::unwrap_err`
    /// isn't available on `parse_fields`'s return type. This is the
    /// manual equivalent.
    fn expect_err(result: syn::Result<(Ident, Vec<FieldInfo>)>) -> syn::Error {
        match result {
            Ok(_) => panic!("expected parse_fields to return an error"),
            Err(err) => err,
        }
    }

    #[test]
    fn enum_input_produces_a_clear_error() {
        let input: DeriveInput = syn::parse_quote! {
            enum Direction { Up, Down }
        };

        let err = expect_err(parse_fields(&input));
        assert!(
            err.to_string().contains("structs with named fields"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn tuple_struct_input_produces_a_clear_error() {
        let input: DeriveInput = syn::parse_quote! {
            struct Point(i32, i32);
        };

        let err = expect_err(parse_fields(&input));
        assert!(
            err.to_string().contains("named fields"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn skip_attribute_is_detected() {
        let input: DeriveInput = syn::parse_quote! {
            struct Config {
                host: String,
                #[summary(skip)]
                secret: String,
            }
        };

        let (name, fields) = parse_fields(&input).expect("valid struct should parse");
        assert_eq!(name, "Config");
        assert!(!fields[0].skip);
        assert!(fields[1].skip);
    }
}
