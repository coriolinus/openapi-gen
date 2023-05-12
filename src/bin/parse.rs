use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use openapiv3::OpenAPI;

#[derive(Debug, Parser)]
struct Args {
    /// path to openapi specification file
    path: PathBuf,

    /// if set, emit debug information for the parsed struct
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let reader = {
        let file = std::fs::File::open(args.path).context("reading file")?;
        std::io::BufReader::new(file)
    };
    let spec: OpenAPI = serde_yaml::from_reader(reader).context("parsing yaml")?;
    if args.debug {
        println!("{spec:#?}");
    }

    Ok(())
}
