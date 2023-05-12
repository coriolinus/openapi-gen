use heck::AsUpperCamelCase;
use openapiv3::{MediaType, ReferenceOr};

pub fn get_ident(prefix: &str, mime_type: &str, media_type: &MediaType) -> String {
    let unknown = || format!("{prefix} {mime_type}");
    let fragment = match media_type.schema.as_ref() {
        // form an identifier like `RenderTemplatePostApplicationJson` if no schema is present
        None => unknown(),
        // form an identifier like `TemplateRequest` if a named schema is used
        Some(ReferenceOr::Reference { reference }) => {
            reference.rsplit('/').next().unwrap_or(reference).to_owned()
        }
        // form an identifier from the `name` extension of the schema,
        // or the `title` field of the schema, or fall back to
        // `RenderTemplatePostApplicationJson`
        Some(ReferenceOr::Item(schema)) => schema
            .schema_data
            .extensions
            .get("name")
            .and_then(|value| value.as_str())
            .or(schema.schema_data.title.as_deref())
            .map(|s| s.to_owned())
            .unwrap_or_else(unknown),
    };
    format!("{}", AsUpperCamelCase(fragment))
}
