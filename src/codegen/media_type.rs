use std::collections::HashSet;

use heck::AsUpperCamelCase;
use indexmap::IndexMap;
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

pub enum Cardinality<'a> {
    Zero,
    One(&'a String, &'a MediaType),
    Several(indexmap::map::Iter<'a, String, MediaType>),
}

/// We often want to vary our behavior based on whether there are Zero, One, or
/// Several distinct `MediaType`s defined in one place.
///
/// It turns out to be difficult to handle this actually properly: the `Schema`
/// type doesn't implement the right traits for us to easiliy determine whether
/// or not we have any equivalent definitions. However, we can make a
/// best-effort guess based on the assumption that programmers will tend not to
/// redundantly declare identical definitions instead of factoring them out.
pub fn distinct(map: &IndexMap<String, MediaType>) -> Cardinality {
    let n_distinct_schema_refs = map
        .values()
        .filter_map(|media_type| {
            media_type
                .schema
                .as_ref()
                .and_then(|schemaref| schemaref.as_ref_str())
        })
        .take(2)
        .collect::<HashSet<_>>()
        .len();

    let n_distinct_local_schemas = map
        .values()
        .filter(|media_type| {
            media_type
                .schema
                .as_ref()
                .map(|schemaref| schemaref.as_ref_str().is_none())
                .unwrap_or_default()
        })
        .take(2)
        .count();

    match n_distinct_schema_refs + n_distinct_local_schemas {
        0 => Cardinality::Zero,
        1 => {
            let (s, m) = map.first().unwrap();
            Cardinality::One(s, m)
        }
        _ => Cardinality::Several(map.iter()),
    }
}
