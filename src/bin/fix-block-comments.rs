use std::{
    io::{self, BufWriter, LineWriter, Write},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;

use openapi_gen::fix_block_comments::fix_block_comments;

#[derive(Debug, Parser)]
struct Args {
    /// path to output file to fix
    path: PathBuf,

    /// whether or not to edit the file in place
    ///
    /// when set, the file is re-emitted to its original location.
    /// otherwise, the result is emitted to stdout.
    #[arg(long)]
    in_place: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let data = std::fs::read_to_string(&args.path)?;
    let output: Box<dyn Write> = if args.in_place {
        let file = std::fs::File::create(&args.path)?;
        Box::new(BufWriter::new(file))
    } else {
        let stdout = io::stdout().lock();
        Box::new(LineWriter::new(stdout))
    };

    fix_block_comments(&data, output)?;

    Ok(())
}
