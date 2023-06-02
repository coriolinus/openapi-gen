use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use openapi_gen::{ApiModel, Error};
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

    let model = ApiModel::try_from(spec).context("converting to api model")?;
    let pretty = model
        .emit_items()
        .map_err(|err| {
            if let Error::CodegenParse { buffer, .. } = &err {
                eprintln!("==== invalid rust code follows ====");
                eprintln!("{buffer}");
                eprintln!("==== invalid rust code precedes ====");
            }
            err
        })
        .context("emitting rust code")?;

    println!("{pretty}");

    Ok(())
}
