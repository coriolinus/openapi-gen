//! Test a dynamic set of example cases against the expected generated code.
//!
//! Each test case is a subfolder of `openapi_gen/tests/cases`, and contains two files:
//! `definition.yaml` and `expect.rs`. The former contains a valid OpenAPI specification,
//! and the latter contains the code which is expected to be generated.
//!
//! Because generated code can vary based on what features are enabled, these tests are disabled
//! unless all such features are enabled. For running this test, the simplest shorthand it just
//! `cargo test --all-features`.
//!
//! Additional debugging information can sometimes be helpful. This can be controlled by setting
//! certain environment variables to any non-empty value.
//!
//! - `DBG_DEFINITION`: emit `dbg!(definition)` for cases which fail.
//! - `DBG_MODEL`: emit `dbg!(model)` for cases which fail.

use std::{
    any::Any,
    fmt,
    io::{IsTerminal, Write},
    path::{Path, PathBuf},
};

use openapi_gen::{ApiModel, Error};
use openapiv3::OpenAPI;
use similar::{ChangeTag, TextDiff};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

#[derive(Debug, derive_more::From)]
enum Outcome {
    Ok,
    Mismatch {
        expect: String,
        have: String,
    },
    Panic(Box<dyn 'static + Any + Send>),
    Error {
        err: anyhow::Error,
        generated_code: String,
    },
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Ok => f.write_str("ok"),
            Outcome::Mismatch { .. } => f.write_str("mismatch"),
            Outcome::Panic(_) => f.write_str("panic"),
            Outcome::Error { .. } => f.write_str("error"),
        }
    }
}

impl Outcome {
    fn is_ok(&self) -> bool {
        matches!(self, Outcome::Ok)
    }

    fn color_spec(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(self.fg_color())).set_bg(self.bg_color());
        spec
    }

    fn fg_color(&self) -> Color {
        match self {
            Outcome::Ok => Color::Green,
            Outcome::Mismatch { .. } | Outcome::Panic(_) | Outcome::Error { .. } => Color::Red,
        }
    }

    fn bg_color(&self) -> Option<Color> {
        match self {
            Outcome::Ok | Outcome::Mismatch { .. } => None,
            Outcome::Panic(_) => Some(Color::Yellow),
            Outcome::Error { .. } => Some(Color::Cyan),
        }
    }

    fn additional_info(&self, out: &mut StandardStream) {
        match self {
            Outcome::Ok | Outcome::Panic(_) => (),
            Outcome::Error {
                err,
                generated_code,
            } => {
                for (idx, err) in err.chain().enumerate() {
                    let _ = writeln!(out, "{idx:>2}: {err}");
                }
                let _ = writeln!(out, "\n{generated_code}");
            }
            Outcome::Mismatch { expect, have } => {
                let diff = TextDiff::from_lines(expect, have);
                for change in diff.iter_all_changes() {
                    let old_line_no = change
                        .old_index()
                        .map(|idx| idx.to_string())
                        .unwrap_or_default();
                    let new_line_no = change
                        .new_index()
                        .map(|idx| idx.to_string())
                        .unwrap_or_default();
                    let _ = write!(out, "{old_line_no:>4} | {new_line_no:>4} ");

                    let tag = match change.tag() {
                        ChangeTag::Equal => " ",
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                    };

                    let color = match change.tag() {
                        ChangeTag::Equal => None,
                        ChangeTag::Delete => Some(Color::Red),
                        ChangeTag::Insert => Some(Color::Green),
                    };

                    let mut spec = ColorSpec::new();
                    spec.set_fg(color);

                    let _ = out.set_color(&spec);
                    let _ = write!(
                        out,
                        "{tag} {}",
                        change.as_str().expect("input was a string")
                    );
                    let _ = out.reset();
                }
            }
        }
    }
}

#[derive(Debug)]
struct Case {
    name: String,
    definition_path: PathBuf,
    definition: OpenAPI,
    expect: syn::File,
}

impl Case {
    fn load_path(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();

        let name = path.file_name()?.to_string_lossy().into_owned();
        let definition_path = path.join("definition.yaml");

        let definition = std::fs::read_to_string(&definition_path).ok()?;
        let definition = serde_yaml::from_str(&definition).ok()?;

        let expect = std::fs::read_to_string(path.join("expect.rs")).ok()?;
        let expect = syn::parse_str(&expect).ok()?;

        Some(Case {
            name,
            definition,
            definition_path,
            expect,
        })
    }

    fn execute(&self) -> Outcome {
        fn execute_inner(
            definition: OpenAPI,
            path: impl AsRef<Path>,
        ) -> Result<syn::File, (anyhow::Error, String)> {
            let model = ApiModel::new(&definition, Some(path))
                .map_err(|err| (err.into(), String::new()))?;
            let pretty = model.emit_items(false).map_err(|err| {
                let buffer = if let Error::CodegenParse { buffer, .. } = &err {
                    buffer.clone()
                } else {
                    String::new()
                };
                (err.into(), buffer)
            })?;
            let file = syn::parse_str::<syn::File>(&pretty).map_err(|err| (err.into(), pretty))?;
            Ok(file)
        }

        let generated = match std::panic::catch_unwind(|| {
            execute_inner(self.definition.clone(), &self.definition_path)
        }) {
            Ok(Ok(generated)) => generated,
            Ok(Err((err, generated_code))) => {
                return Outcome::Error {
                    err,
                    generated_code,
                }
            }
            Err(panic) => return Outcome::Panic(panic),
        };

        if generated == self.expect {
            Outcome::Ok
        } else {
            Outcome::Mismatch {
                expect: prettyplease::unparse(&self.expect),
                have: prettyplease::unparse(&generated),
            }
        }
    }
}

fn find_cases() -> impl Iterator<Item = Case> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cases");
    std::fs::read_dir(path)
        .expect("cases subfolder of tests directory must exist")
        .filter_map(|maybe_dir_entry| maybe_dir_entry.ok())
        .filter(|dir_entry| {
            dir_entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or_default()
        })
        .filter_map(|dir_entry| Case::load_path(dir_entry.path()))
}

fn env_is_set(name: impl AsRef<std::ffi::OsStr>) -> bool {
    std::env::var_os(name)
        .map(|value| !value.is_empty())
        .unwrap_or_default()
}

#[test]
#[cfg_attr(
    not(all(
        feature = "api-problem",
        feature = "bytes",
        feature = "integer-restrictions",
        feature = "string-pattern"
    )),
    ignore = "required features are not enabled"
)]
fn cases() {
    let choice = if std::io::stdout().is_terminal() {
        termcolor::ColorChoice::Auto
    } else {
        termcolor::ColorChoice::Never
    };
    let mut stdout = StandardStream::stdout(choice);

    let mut all_ok = true;
    let mut cases_run = 0;

    // later we can parallelize this, but for now, straight iteration should be totally fine
    for case in find_cases() {
        cases_run += 1;

        let _ = write!(&mut stdout, "{} ... ", &case.name);
        let _ = stdout.flush();

        let outcome = case.execute();
        all_ok &= matches!(outcome, Outcome::Ok);
        let color_spec = outcome.color_spec();
        let _ = stdout.set_color(&color_spec);

        let _ = writeln!(&mut stdout, "{outcome}");
        let _ = stdout.reset();

        outcome.additional_info(&mut stdout);
        if !outcome.is_ok() {
            if env_is_set("DBG_DEFINITION") {
                dbg!(&case.definition);
            }
            if env_is_set("DBG_MODEL") {
                if let Ok(model) = ApiModel::new(&case.definition, Some(case.definition_path)) {
                    dbg!(model);
                } else {
                    eprintln!("failed to parse ApiModel from OpenAPI definition");
                }
            }
        }
    }

    if !all_ok {
        panic!("not all cases passed")
    }
    if cases_run == 0 {
        panic!("no test cases discovered")
    }
}
