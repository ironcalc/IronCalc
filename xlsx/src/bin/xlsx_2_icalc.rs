//! Tests an Excel xlsx file.
//! Returns a list of differences in json format.
//! Saves an `IronCalc` version
//! This is primary for QA internal testing and will be superseded by an official
//! `IronCalc` CLI.
//!
//! Usage: test file.xlsx

use anyhow::{bail, Context, Result};
use clap::Parser;
use ironcalc::{
    export::{save_to_icalc, ModelType},
    import::{load_from_csv, load_from_xlsx},
};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// file to convert
    path: PathBuf,

    /// kind of file
    #[clap(short, value_enum)]
    kind: Option<FileKind>,

    /// output file path, will match path with `ic` extension otherwise
    #[clap(short, long)]
    output: Option<PathBuf>,
}

/// The kind of file that is being requested
#[derive(clap::ValueEnum, Clone, Debug)]
enum FileKind {
    Xlsx,
    Csv,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let path = args.path;

    let kind = match args.kind {
        Some(kind) => kind,
        None => get_file_kind(&path)?,
    };

    let output_path = create_output_path(args.output, &path)?;

    let model = match kind {
        FileKind::Xlsx => handle_xlsx(&path),
        FileKind::Csv => handle_csv(&path),
    }?;

    save_to_icalc(&model, &output_path).with_context(|| "Failed to sasve file as .icalc")?;
    Ok(())
}

fn create_output_path(out_path: Option<PathBuf>, in_path: &Path) -> Result<String> {
    let output_path: PathBuf = if let Some(out) = out_path {
        out
    } else {
        let base_name = in_path.file_stem();

        match base_name {
            Some(base_name) => PathBuf::from(base_name).with_extension("ic"),
            None => {
                bail!("Issue finding file stem of path: {}", in_path.display())
            }
        }
    };

    let output_file_name: String = output_path
        .to_str()
        .context(format!("Issue with path: {output_path:?}"))?
        .to_owned();
    Ok(output_file_name)
}

fn get_file_kind(path: &Path) -> anyhow::Result<FileKind> {
    use FileKind::{Csv, Xlsx};

    let extension = path.extension();

    let kind = if let Some(extension) = extension {
        let extension = extension.to_str().context(format!(
            "There was an issue with the provided path and the system string type. path: {}",
            path.display()
        ))?;
        match extension.to_ascii_lowercase().as_str() {
            "xlsx" => Xlsx,
            "csv" => Csv,
            _ => bail!(
                "Unsupported auto-detected extension of: {}, found on provided path: {}",
                extension,
                path.display()
            ),
        }
    } else {
        bail!(
            "Could not detect extension on provided path: {}",
            path.display()
        );
    };

    Ok(kind)
}

fn handle_xlsx(path: &Path) -> Result<ModelType> {
    let Some(path) = path.to_str() else {
        bail!("Could not parse provided path: {}", path.display())
    };
    let model = load_from_xlsx(path, "en", "UTC")?;
    Ok(ModelType::Model(model))
}

fn handle_csv(path: &Path) -> Result<ModelType> {
    // TODO: add support for manual CSV parsing settings instead of automatic
    let model = load_from_csv(path)?;
    Ok(ModelType::UserModel(model))
}
