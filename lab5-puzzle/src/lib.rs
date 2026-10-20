//! WireSummary: parse policy, validate labels, bound visible fields, emit code.
//! The starter emits a compiling method that panics when called.
#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use syn::ext::IdentExt;
use syn::{DeriveInput, Field, Generics, Ident, LitStr, Type};

#[derive(Default)]
struct Options {
    skip: bool,
    redact: bool,
    rename: Option<LitStr>,
}
struct FieldInfo {
    ident: Ident,
    ty: Type,
    label: LitStr,
    options: Options,
}

// A: parse all summary attributes in source order. Unknown, duplicate or
// conflicting options are errors, not requests to silently fall back.
fn parse_options(field: &Field) -> syn::Result<Options> {
    // TODO A BEGIN
    let _ = field;
    Ok(Options::default())
    // TODO A END
}

// Supplied: accept named-field structs, preserve field types and source spans.
fn collect_fields(input: &DeriveInput) -> syn::Result<Vec<FieldInfo>> {
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "WireSummary requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "WireSummary requires a struct with named fields",
            ));
        }
    };
    fields
        .iter()
        .map(|field| {
            let ident = field.ident.clone().expect("named field");
            let options = parse_options(field)?;
            let label = options
                .rename
                .clone()
                .unwrap_or_else(|| LitStr::new(&ident.unraw().to_string(), ident.span()));
            Ok(FieldInfo {
                ident,
                ty: field.ty.clone(),
                label,
                options,
            })
        })
        .collect()
}

// B: final output labels must be unique. Skipped fields do not reserve a label.
// Return an error at the conflicting label, including "duplicate output label".
fn validate_fields(fields: &[FieldInfo]) -> syn::Result<()> {
    // TODO B BEGIN
    let _ = fields;
    Ok(())
    // TODO B END
}

// C: preserve lifetimes, const parameters, defaults and the existing where
// clause. Add a Debug predicate for each VISIBLE field TYPE, not every generic.
// Skipped and redacted fields must impose no Debug requirement.
fn bounded_generics(input: &DeriveInput, fields: &[FieldInfo]) -> Generics {
    // TODO C BEGIN
    let _ = fields;
    input.generics.clone()
    // TODO C END
}

// D: emit an inherent wire_summary(&self) -> String using the bounded generics.
// Visible fields use Debug once; redacted fields emit the literal <redacted>
// WITHOUT evaluating that field's Debug implementation. Skipped fields vanish.
// Qualify standard-library paths so caller types named String/Vec do not break
// expansion. Keep input syntax out of manually concatenated Rust source text.
fn emit_summary(input: &DeriveInput, fields: &[FieldInfo]) -> Tokens {
    // TODO D BEGIN
    let name = &input.ident;
    let generics = bounded_generics(input, fields);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics #name #type_generics #where_clause {
            pub fn wire_summary(&self) -> ::std::string::String {
                ::std::unimplemented!("D: emit the summary without exposing hidden fields")
            }
        }
    }
    // TODO D END
}

#[proc_macro_derive(WireSummary, attributes(summary))]
pub fn wire_summary(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let result = collect_fields(&input).and_then(|fields| {
        validate_fields(&fields)?;
        Ok(emit_summary(&input, &fields))
    });
    result.unwrap_or_else(syn::Error::into_compile_error).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields(source: &str) -> Vec<FieldInfo> {
        collect_fields(&syn::parse_str(source).unwrap()).unwrap()
    }
    #[test]
    fn parser_combines_separate_helper_attributes() {
        let fields =
            fields(r#"struct Packet { #[summary(rename="secret")] #[summary(redact)] token: u8 }"#);
        assert!(fields[0].options.redact);
        assert_eq!(fields[0].label.value(), "secret");
    }
    #[test]
    fn parser_rejects_duplicate_and_conflicting_options() {
        for source in [
            "struct A { #[summary(skip, skip)] x: u8 }",
            "struct A { #[summary(skip)] #[summary(redact)] x: u8 }",
            "struct A { #[summary(unknown)] x: u8 }",
            "struct A { #[summary(rename=\"\")] x: u8 }",
        ] {
            let input = syn::parse_str(source).unwrap();
            assert!(collect_fields(&input).is_err(), "{source}");
        }
    }
    #[test]
    fn labels_reject_collisions_but_ignore_skipped_fields() {
        let duplicate = fields(r#"struct A { #[summary(rename="id")] a:u8, id:u8 }"#);
        assert!(validate_fields(&duplicate).is_err());
        let skipped =
            fields("struct A { #[summary(skip)] id:u8, #[summary(rename=\"id\")] other:u8 }");
        assert!(validate_fields(&skipped).is_ok());
    }
    #[test]
    fn bounds_apply_to_visible_field_types_only() {
        let input: DeriveInput = syn::parse_quote! {
            struct A<T, U, V> where V: Clone {
                shown: Option<T>,
                #[summary(skip)] ignored: U,
                #[summary(redact)] secret: V,
            }
        };
        let fields = collect_fields(&input).unwrap();
        let bounded = bounded_generics(&input, &fields);
        let predicates = &bounded.where_clause.unwrap().predicates;
        let actual = quote!(#predicates).to_string();
        assert!(
            actual.contains("Option < T > : :: std :: fmt :: Debug"),
            "{actual}"
        );
        assert!(!actual.contains("U : :: std :: fmt :: Debug"), "{actual}");
        assert!(!actual.contains("V : :: std :: fmt :: Debug"), "{actual}");
        assert!(actual.contains("V : Clone"), "{actual}");
    }
}
