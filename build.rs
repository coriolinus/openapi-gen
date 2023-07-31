fn main() -> Result<(), Box<dyn std::error::Error>> {
    // set the environment variable `VERGEN_GIT_SHA` to the current commit's sha
    // at build time, so it can be emitted in output file headers
    vergen::EmitBuilder::builder().git_sha(true).emit()?;
    Ok(())
}
