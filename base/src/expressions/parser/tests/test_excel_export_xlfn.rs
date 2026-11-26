use std::collections::HashMap;

use crate::expressions::{
    parser::{stringify::to_excel_string, tests::utils::new_parser},
    types::CellReferenceRC,
};

fn excel_export(formula: &str) -> String {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let t = parser.parse(formula, &cell_reference);
    to_excel_string(&t, &cell_reference)
}

// These are stored by Excel without the `_xlfn.` prefix.
// CEILING and GAMMALN are plain built-ins; ECMA.CEILING and ISO.CEILING are
// listed in [MS-XLSX] 2.2.3 without it. Checked against a file saved by Excel.
#[test]
fn functions_without_xlfn_prefix() {
    for formula in [
        "CEILING(12.34435,2)",
        "GAMMALN(3)",
        "ECMA.CEILING(12.12324,3)",
        "ISO.CEILING(12.4466)",
    ] {
        assert_eq!(excel_export(formula), formula);
        // The prefixed spelling is still accepted on import
        assert_eq!(excel_export(&format!("_xlfn.{formula}")), formula);
    }
}

// These are future functions, Excel stores them with the `_xlfn.` prefix.
// All but FORECAST.LINEAR were checked against a file saved by Excel,
// FORECAST.LINEAR follows [MS-XLSX] 2.2.3.
#[test]
fn functions_with_xlfn_prefix() {
    for formula in [
        "DAYS(15,18)",
        "GAMMA.DIST(3,1,1,TRUE)",
        "GAUSS(3)",
        "IMSECH(12)",
        "FORECAST.LINEAR(1,A1:A3,B1:B3)",
    ] {
        let expected = format!("_xlfn.{formula}");
        assert_eq!(excel_export(formula), expected);
        assert_eq!(excel_export(&expected), expected);
    }
}
