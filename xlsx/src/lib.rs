//! This cate reads an xlsx file and transforms it into an internal representation ([`Model`]).
//! An `xlsx` is a zip file containing a set of folders and `xml` files. The IronCalc json structure mimics the relevant parts of the Excel zip.
//! Although the xlsx structure is quite complicated, it's essentials regarding the spreadsheet technology are easier to grasp.
//!
//! The simplest workbook folder structure might look like this:
//!
//! ```text
//! docProps
//!     app.xml
//!     core.xml
//!
//! _rels
//!     .rels
//!
//! xl
//!     _rels
//!         workbook.xml.rels
//!     theme
//!         theme1.xml
//!     worksheets
//!         sheet1.xml
//!     calcChain.xml
//!     styles.xml
//!     workbook.xml
//!     sharedStrings.xml
//!
//! [Content_Types].xml
//! ```
//!
//! Note that more complicated workbooks will have many more files and folders.
//! For instance charts, pivot tables, comments, tables,...
//!
//! The relevant json structure in IronCalc will be:
//!
//! ```json
//! {
//!     "name": "Workbook1",
//!     "defined_names": [],
//!     "shared_strings": [],
//!     "worksheets": [],
//!     "styles": {
//!         "num_fmts": [],
//!         "fonts": [],
//!         "fills": [],
//!         "borders": [],
//!         "cell_style_xfs": [],
//!         "cell_styles" : [],
//!         "cell_xfs": []
//!     }
//! }
//! ```
//!
//! Note that there is not a 1-1 correspondence but there is a close resemblance.
//!
//! [`Model`]: ../ironcalc/struct.Model.html

pub mod compare;
pub mod error;
pub mod export;
pub mod import;
pub use ironcalc_base as base;
