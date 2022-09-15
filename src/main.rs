use anyhow::{anyhow, bail, Result};
use clap::Parser;
use std::{collections::BTreeMap, fs, io, path::PathBuf, process::Command};
use wasmtime::{Config, Engine, Module};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Options {
    /// Directory to scan for modules.
    ///
    /// Each subdirectory of this directory will be assumed to contain a project with a makefile that produces a
    /// file named "spin.wasm".  These files will be evaluated using `spin_conformance::test` to produce a matrix
    /// of results.
    #[clap(default_value = "./modules")]
    pub module_directory: PathBuf,
}

fn main() -> Result<()> {
    let options = &Options::parse();

    let engine = &Engine::new(&Config::new())?;

    let mut matrix = BTreeMap::new();

    for entry in fs::read_dir(&options.module_directory)? {
        let path = entry?.path();
        if path.is_dir() {
            let status = Command::new("make")
                .current_dir(&path)
                .args(["spin.wasm"])
                .status()?;

            if status.success() {
                let config =
                    if let Ok(config) = fs::read_to_string(&path.join("spin-conformance.toml")) {
                        toml::from_str(&config)?
                    } else {
                        spin_conformance::Config::default()
                    };

                matrix.insert(
                    path.file_name()
                        .ok_or_else(|| {
                            anyhow!(
                                "unnamed directory in {}",
                                options.module_directory.display()
                            )
                        })?
                        .to_string_lossy()
                        .into_owned(),
                    spin_conformance::test(
                        &Module::from_file(engine, &path.join("spin.wasm"))?,
                        config,
                    )?,
                );
            } else {
                bail!("`make` command failed");
            }
        }
    }

    serde_json::to_writer_pretty(io::stdout().lock(), &matrix)?;

    Ok(())
}
