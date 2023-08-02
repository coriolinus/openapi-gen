use crate::codegen::make_ident;

use heck::ToUpperCamelCase;
use openapiv3::{AnySchema, Schema};
use proc_macro2::TokenStream;
use quote::quote;

/// OpenAPI's string `enum` type.
///
/// Also covers `x-extensible-enum`.
#[derive(Debug, Clone)]
pub struct StringEnum {
    pub variants: Vec<String>,
    pub extensible: bool,
}

impl StringEnum {
    /// OpenAPI has a somewhat silly rule: to define a nullable enum, you must explicitly include `null` among the
    /// stated enum variants. The OpenAPI schema model we're using doesn't handle that case well; see
    /// <https://github.com/kurtbuilds/openapiv3/issues/3>.
    ///
    /// This function tries to work around that problem.
    pub(crate) fn new(schema: &Schema, any_schema: &AnySchema) -> Option<Self> {
        use serde_json::Value;

        // This function is implemented as a giant `Option` combinator.
        //
        // It's formed of two branches, each of which is of the form
        // `condition.then(|| make_a_some_variant())`.

        let schema_data = &schema.schema_data;
        (schema_data.nullable
            && any_schema.typ.as_deref() == Some("string")
            && any_schema
                .enumeration
                .iter()
                .any(|enum_item| matches!(enum_item, Value::Null))
            && any_schema.enumeration.iter().all(|enum_item| {
                matches!(enum_item, Value::Null) || matches!(enum_item, Value::String(_))
            }))
        .then(|| StringEnum {
            variants: any_schema
                .enumeration
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect(),
            extensible: false,
        })
        .or_else(|| {
            const X_EXTENSIBLE_ENUM: &str = "x-extensible-enum";
            (schema_data.nullable
                && any_schema.typ.as_deref() == Some("string")
                && schema_data
                    .extensions
                    .get(X_EXTENSIBLE_ENUM)
                    .and_then(Value::as_array)
                    .map(|array| {
                        array.iter().any(|value| matches!(value, Value::Null))
                            && array.iter().all(|value| {
                                matches!(value, Value::Null) || matches!(value, Value::String(_))
                            })
                    })
                    .unwrap_or_default())
            .then(|| StringEnum {
                variants: schema_data
                    .extensions
                    .get(X_EXTENSIBLE_ENUM)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect(),
                extensible: true,
            })
        })
    }

    pub fn emit_definition(&self) -> TokenStream {
        let mut variants = self
            .variants
            .iter()
            .map(|variant| {
                let ident = make_ident(&variant.to_upper_camel_case());
                quote!(#[serde(rename = #variant)] #ident)
            })
            .collect::<Vec<_>>();
        if self.extensible {
            // Normally, we're just going to call the "other" field "Other", but
            // in case there's a conflict, ensure it's unique. We're not too
            // concerned about efficiency here; this should only very rarely loop
            // more than once or twice.
            let mut other_name = "Other".to_string();
            while self.variants.contains(&other_name) {
                other_name.push('_');
            }
            let other_name = make_ident(&other_name);
            variants.push(quote!(#[serde(other)] #other_name(String)));
        }
        quote! {
            { #( #variants ),* }
        }
    }

    pub(crate) fn impls_copy(&self) -> bool {
        !self.extensible
    }
}
