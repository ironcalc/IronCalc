use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PDF generation error: {0}")]
    Pdf(String),
    #[error("Sheet not found: {0}")]
    SheetNotFound(String),
    #[error("Invalid range: {0}")]
    InvalidRange(String),
    #[error("IronCalc error: {0}")]
    IronCalc(String),
}
