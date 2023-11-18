#![allow(clippy::unwrap_used)]

use crate::expressions::types::{Area, CellReferenceIndex};
use crate::test::util::new_empty_model;

#[test]
fn test_forward_references() {
    let mut model = new_empty_model();

    // test single ref changed nd not changed
    model._set("H8", "=F6*G9");
    // tests areas
    model._set("H9", "=SUM(D4:F6)");
    // absolute coordinates
    model._set("H10", "=$F$6");
    // area larger than the source area
    model._set("H11", "=SUM(D3:F6)");
    // Test arguments and concat
    model._set("H12", "=SUM(F6, D4:F6) & D4");
    // Test range operator. This is syntax error for now.
    // model._set("H13", "=SUM(D4:INDEX(D4:F5,4,2))");
    // Test operations
    model._set("H14", "=-D4+D5*F6/F5");

    model.evaluate();

    // Source Area is D4:F6
    let source_area = &Area {
        sheet: 0,
        row: 4,
        column: 4,
        width: 3,
        height: 3,
    };

    // We paste in B10
    let target_row = 10;
    let target_column = 2;
    let result = model.forward_references(
        source_area,
        &CellReferenceIndex {
            sheet: 0,
            row: target_row,
            column: target_column,
        },
    );
    assert!(result.is_ok());
    model.evaluate();

    assert_eq!(model._get_formula("H8"), "=D12*G9");
    assert_eq!(model._get_formula("H9"), "=SUM(B10:D12)");
    assert_eq!(model._get_formula("H10"), "=$D$12");

    assert_eq!(model._get_formula("H11"), "=SUM(D3:F6)");
    assert_eq!(model._get_formula("H12"), "=SUM(D12,B10:D12)&B10");
    // assert_eq!(model._get_formula("H13"), "=SUM(B10:INDEX(B10:D11,4,2))");
    assert_eq!(model._get_formula("H14"), "=-B10+B11*D12/D11");
}

#[test]
fn test_different_sheet() {
    let mut model = new_empty_model();

    // test single ref changed not changed
    model._set("H8", "=F6*G9");
    // tests areas
    model._set("H9", "=SUM(D4:F6)");
    // absolute coordinates
    model._set("H10", "=$F$6");
    // area larger than the source area
    model._set("H11", "=SUM(D3:F6)");
    // Test arguments and concat
    model._set("H12", "=SUM(F6, D4:F6) & D4");
    // Test range operator. This is syntax error for now.
    // model._set("H13", "=SUM(D4:INDEX(D4:F5,4,2))");
    // Test operations
    model._set("H14", "=-D4+D5*F6/F5");

    // Adds a new sheet
    assert!(model.add_sheet("Sheet2").is_ok());

    model.evaluate();

    // Source Area is D4:F6
    let source_area = &Area {
        sheet: 0,
        row: 4,
        column: 4,
        width: 3,
        height: 3,
    };

    // We paste in Sheet2!B10
    let target_row = 10;
    let target_column = 2;
    let result = model.forward_references(
        source_area,
        &CellReferenceIndex {
            sheet: 1,
            row: target_row,
            column: target_column,
        },
    );
    assert!(result.is_ok());
    model.evaluate();

    assert_eq!(model._get_formula("H8"), "=Sheet2!D12*G9");
    assert_eq!(model._get_formula("H9"), "=SUM(Sheet2!B10:D12)");
    assert_eq!(model._get_formula("H10"), "=Sheet2!$D$12");

    assert_eq!(model._get_formula("H11"), "=SUM(D3:F6)");
    assert_eq!(
        model._get_formula("H12"),
        "=SUM(Sheet2!D12,Sheet2!B10:D12)&Sheet2!B10"
    );
    // assert_eq!(model._get_formula("H13"), "=SUM(B10:INDEX(B10:D11,4,2))");
    assert_eq!(
        model._get_formula("H14"),
        "=-Sheet2!B10+Sheet2!B11*Sheet2!D12/Sheet2!D11"
    );
}
