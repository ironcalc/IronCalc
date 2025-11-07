#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_average_accrint_simple_cases() {
    let mut model = new_empty_model();
    // ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])
    model._set("A1", "=ACCRINT(39508, 39691, 39569, 0.1, 1000, 2, 0)");
    model._set(
        "A2",
        "=ACCRINT(DATE(2008, 3, 5), 39691, 39569, 0.1, 1000, 2, 0, FALSE)",
    );
    model._set(
        "A3",
        "=ACCRINT(DATE(2008, 4, 5), 39691, 39569, 0.1, 1000, 2, 0, TRUE)",
    );
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"16.666666667");
    assert_eq!(model._get_text("A2"), *"15.555555556");
    assert_eq!(model._get_text("A3"), *"7.222222222");
}
