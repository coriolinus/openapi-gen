use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use openapi_gen::{ApiModel, Error};
use openapiv3::OpenAPI;

#[derive(Debug, Parser)]
struct Args {
    /// path to openapi specification file
    path: PathBuf,

    /// emit debug information about the spec
    ///
    /// when set, this suppresses emitting the normal generated rust code.
    /// to override this, set `--emit-rust`.
    #[arg(long)]
    debug_spec: bool,

    /// emit debug information about the model
    ///
    /// when set, this suppresses emitting the normal generated rust code.
    /// to override this, set `--emit-rust`.
    #[arg(long)]
    debug_model: bool,

    /// force emitting generated rust code
    ///
    /// this is normally not required, but the generated code is suppressed by default
    /// when `--debug-spec` or `--debug-model` is used.
    #[arg(long)]
    emit_rust: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let reader = {
        let file = std::fs::File::open(args.path).context("reading file")?;
        std::io::BufReader::new(file)
    };
    let spec: OpenAPI = serde_yaml::from_reader(reader).context("parsing yaml")?;

    let model = ApiModel::try_from(spec.clone()).context("converting to api model")?;
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

    if args.debug_spec {
        dbg!(&spec);
    }
    if args.debug_model {
        dbg!(&model);
    }
    if args.emit_rust || !(args.debug_spec || args.debug_model) {
        println!("{pretty}");
    }

    Ok(())
}
