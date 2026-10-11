//! # IronCalc PDF Export
//!
//! A Rust-native spreadsheet-to-PDF export library for IronCalc.
//! this crate provides an intermediate print layout engine and vector PDF generation pipeline

pub mod error;
pub mod layout;
pub mod options;
pub mod renderer;

pub use error::PdfError;
pub use layout::{plan_export_layout, PagePlan};
pub use options::*;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use ironcalc_base::Model;

/// Exports a workbook model to a PDF byte vector according to the provided options.
///
/// # Example
///
/// ```rust,no_run
/// use ironcalc_base::Model;
/// use ironcalc_pdf::{export_to_pdf, PdfExportOptions};
///
/// let model = Model::new_empty("Sheet1", "en", "UTC", "en").unwrap();
/// let options = PdfExportOptions::default().fit_to_page();
/// let pdf_bytes = export_to_pdf(&model, &options).unwrap();
/// assert!(!pdf_bytes.is_empty());
/// ```
pub fn export_to_pdf(model: &Model, options: &PdfExportOptions) -> Result<Vec<u8>, PdfError> {
    let plans = layout::plan_export_layout(model, options)?;
    renderer::render_pdf(model, options, &plans)
}

/// Exports a workbook model directly to a PDF file at the specified path.
pub fn export_to_pdf_file(
    model: &Model,
    options: &PdfExportOptions,
    path: impl AsRef<Path>,
) -> Result<(), PdfError> {
    let bytes = export_to_pdf(model, options)?;
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}
