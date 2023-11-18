use std::io;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum XlsxError {
    #[error("I/O Error: {0}")]
    IO(String),
    #[error("Zip Error: {0}")]
    Zip(String),
    #[error("XML Error: {0}")]
    Xml(String),
    #[error("{0}")]
    Workbook(String),
    #[error("Evaluation Error: {}", .0.join("; "))]
    Evaluation(Vec<String>),
    #[error("Comparison Error: {0}")]
    Comparison(String),
    #[error("Not Implemented Error: {0}")]
    NotImplemented(String),
}

impl From<io::Error> for XlsxError {
    fn from(error: io::Error) -> Self {
        XlsxError::IO(error.to_string())
    }
}

impl From<ZipError> for XlsxError {
    fn from(error: ZipError) -> Self {
        XlsxError::Zip(error.to_string())
    }
}

impl From<ParseIntError> for XlsxError {
    fn from(error: ParseIntError) -> Self {
        XlsxError::Xml(error.to_string())
    }
}

impl From<ParseFloatError> for XlsxError {
    fn from(error: ParseFloatError) -> Self {
        XlsxError::Xml(error.to_string())
    }
}

impl From<roxmltree::Error> for XlsxError {
    fn from(error: roxmltree::Error) -> Self {
        XlsxError::Xml(error.to_string())
    }
}

impl XlsxError {
    pub fn user_message(&self) -> String {
        match &self {
            XlsxError::IO(_) | XlsxError::Workbook(_) => self.to_string(),
            XlsxError::Zip(_) | XlsxError::Xml(_) => {
                "IronCalc can only open workbooks created by Microsoft Excel. \
                Can you open this file with Excel, save it to a new file, \
                and then open that new file with IronCalc? If you've already tried this, \
                then send this workbook to support@ironcalc.com and our engineering team \
                will work with you to fix the issue."
                    .to_string()
            }
            XlsxError::NotImplemented(error) => format!(
                "IronCalc cannot open this workbook due to the following unsupported features: \
                {error}. You can either re-implement these parts of your workbook using features \
                supported by IronCalc, or you can send this workbook to support@ironcalc.com \
                and our engineering team will work with you to fix the issue.",
            ),
            XlsxError::Evaluation(errors) => format!(
                "IronCalc could not evaluate this workbook without errors. This may indicate a bug or missing feature \
                in the IronCalc spreadsheet calculation engine. Please contact support@ironcalc.com, share the entirety \
                of this error message and the relevant workbook, and we will work with you to resolve the issue. \
                Detailed error message:\n{}",
                errors.join("\n")
            ),
            XlsxError::Comparison(error) => format!(
                "IronCalc produces different results when evaluating the workbook \
                than those already present in the workbook. This may indicate a bug or missing \
                feature in the IronCalc spreadsheet calculation engine. Please contact \
                support@ironcalc.com, share the entirety of this error message and the relevant \
                workbook, and we will work with you to resolve the issue. \
                Detailed error message:\n{error}"
            ),
        }
    }
}
