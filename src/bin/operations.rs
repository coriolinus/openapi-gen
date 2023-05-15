use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use openapi_gen::codegen::operation;
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

    for (path, item) in spec.paths.iter() {
        let Some(item) = item.as_item() else {
            bail!("unable to resolve path item: {:?}", item.as_ref_str());
        };

        for (operation_name, operation) in item.iter() {
            let prefix_ident = operation::get_ident(operation_name, path, operation);

            let request_item = operation::make_request_item(&spec, &prefix_ident, operation);
            println!("{request_item}");

            let response_item = operation::make_response_item(&spec, &prefix_ident, operation);
            println!("{response_item}");
        }
    }

    Ok(())
}
