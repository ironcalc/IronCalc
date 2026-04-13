#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_accrintm_microsoft_example() {
    // Microsoft docs: issue=2008-03-01 (39508), settlement=2008-05-15 (39583)
    // Corrected serials: 2008-03-01=39508, but Microsoft uses 39539/39614
    // Using Microsoft's exact serials: 39539=2008-04-01, 39614=2008-06-15
    // rate=0.1, par=1000, basis=3 (Actual/365)
    // 75 actual days, YEARFRAC = 75/365 = 0.205479452055...
    // ACCRINTM = 1000 * 0.1 * 75/365 = 20.547945205...
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, 0.1, 1000, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"20.547945205");
}

#[test]
fn fn_accrintm_basis_0_default() {
    // issue=2008-04-01 (39539), settlement=2008-06-15 (39614)
    // rate=0.1, par=1000, basis=0 (US 30/360)
    // 30/360 day count: (6-4)*30 + (15-1) = 74 days
    // YEARFRAC = 74/360 = 0.205555...
    // ACCRINTM = 1000 * 0.1 * 74/360 = 20.555555...
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, 0.1, 1000, 0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"20.555555556");
}

#[test]
fn fn_accrintm_par_default() {
    // Omit par (default 1000) and basis (default 0)
    // Same as basis_0_default test
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, 0.1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"20.555555556");
}

#[test]
fn fn_accrintm_error_issue_after_settlement() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39614, 39539, 0.1, 1000, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

#[test]
fn fn_accrintm_error_issue_equals_settlement() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39539, 0.1, 1000, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

#[test]
fn fn_accrintm_error_negative_rate() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, -0.05, 1000, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

#[test]
fn fn_accrintm_error_zero_rate() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, 0, 1000, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

#[test]
fn fn_accrintm_error_zero_par() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, 0.1, 0, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

#[test]
fn fn_accrintm_error_invalid_basis() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, 0.1, 1000, 5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}