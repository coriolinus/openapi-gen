use std::fmt::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use openapi_gen::{
    codegen::schema::make_items_for_schema,
    openapi_compat::{component_schemas, operation_inline_schemas, path_operations},
};
use openapiv3::OpenAPI;

#[derive(Debug, Parser)]
struct Args {
    /// path to openapi specification file
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let reader = {
        let file = std::fs::File::open(args.path).context("reading file")?;
        std::io::BufReader::new(file)
    };
    let spec: OpenAPI = serde_yaml::from_reader(reader).context("parsing yaml")?;

    let mut buffer = String::new();

    for maybe_path_operation in path_operations(&spec) {
        let (path, operation_name, operation) = maybe_path_operation?;

        for (derived_name, schema) in operation_inline_schemas(path, operation_name, operation) {
            let derived = make_items_for_schema(&spec, &derived_name, schema);
            writeln!(&mut buffer, "{derived}")?;
        }
    }

    for (name, schema) in component_schemas(&spec) {
        let derived = make_items_for_schema(&spec, name, schema);
        writeln!(&mut buffer, "{derived}")?;
    }

    let file = syn::parse_str::<syn::File>(&buffer)?;
    let pretty = prettyplease::unparse(&file);

    println!("{pretty}");

    Ok(())
}
