#![allow(clippy::unwrap_used)]

use crate::{cell::CellValue, test::util::new_empty_model};

#[test]
fn fn_price_yield() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");
    model._set("A3", "5%");

    model._set("B1", "=PRICE(A1,A2,A3,6%,100,1)");
    model._set("B2", "=YIELD(A1,A2,A3,B1,100,1)");

    model.evaluate();
    assert_eq!(model._get_text("B1"), "99.056603774");
    assert_eq!(model._get_text("B2"), "0.06");
}

#[test]
fn fn_price_frequencies() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=PRICE(A1,A2,5%,6%,100,1)");
    model._set("B2", "=PRICE(A1,A2,5%,6%,100,2)");
    model._set("B3", "=PRICE(A1,A2,5%,6%,100,4)");

    model.evaluate();

    let annual: f64 = model._get_text("B1").parse().unwrap();
    let semi: f64 = model._get_text("B2").parse().unwrap();
    let quarterly: f64 = model._get_text("B3").parse().unwrap();

    assert_ne!(annual, semi);
    assert_ne!(semi, quarterly);
}

#[test]
fn fn_yield_frequencies() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=YIELD(A1,A2,5%,99,100,1)");
    model._set("B2", "=YIELD(A1,A2,5%,99,100,2)");
    model._set("B3", "=YIELD(A1,A2,5%,99,100,4)");

    model.evaluate();

    let annual: f64 = model._get_text("B1").parse().unwrap();
    let semi: f64 = model._get_text("B2").parse().unwrap();
    let quarterly: f64 = model._get_text("B3").parse().unwrap();

    assert_ne!(annual, semi);
    assert_ne!(semi, quarterly);
}

#[test]
fn fn_price_argument_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=PRICE()");
    model._set("B2", "=PRICE(A1,A2,5%,6%,100)");
    model._set("B3", "=PRICE(A1,A2,5%,6%,100,2,0,99)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_yield_argument_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=YIELD()");
    model._set("B2", "=YIELD(A1,A2,5%,99,100)");
    model._set("B3", "=YIELD(A1,A2,5%,99,100,2,0,99)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_price_invalid_frequency() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=PRICE(A1,A2,5%,6%,100,0)");
    model._set("B2", "=PRICE(A1,A2,5%,6%,100,3)");
    model._set("B3", "=PRICE(A1,A2,5%,6%,100,5)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
    assert_eq!(model._get_text("B3"), *"#NUM!");
}

#[test]
fn fn_pricedisc() {
    let mut model = new_empty_model();
    model._set("A2", "=DATE(2022,1,25)");
    model._set("A3", "=DATE(2022,11,15)");
    model._set("A4", "3.75%");
    model._set("A5", "100");

    model._set("B1", "=PRICEDISC(A2,A3,A4,A5)");
    model._set("C1", "=PRICEDISC(A2,A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "96.979166667");
    assert_eq!(model._get_text("C1"), *"#ERROR!");
}

#[test]
fn fn_pricemat() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2019,2,15)");
    model._set("A2", "=DATE(2025,4,13)");
    model._set("A3", "=DATE(2018,11,11)");
    model._set("A4", "5.75%");
    model._set("A5", "6.5%");

    model._set("B1", "=PRICEMAT(A1,A2,A3,A4,A5)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "96.271187821");
}

#[test]
fn fn_yielddisc() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2022,1,25)");
    model._set("A2", "=DATE(2022,11,15)");
    model._set("A3", "97");
    model._set("A4", "100");

    model._set("B1", "=YIELDDISC(A1,A2,A3,A4)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "0.038393175");
}

#[test]
fn fn_yieldmat() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2019,2,15)");
    model._set("A2", "=DATE(2025,4,13)");
    model._set("A3", "=DATE(2018,11,11)");
    model._set("A4", "5.75%");
    model._set("A5", "96.27");

    model._set("B1", "=YIELDMAT(A1,A2,A3,A4,A5)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "0.065002762");
}

#[test]
fn fn_disc() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2022,1,25)");
    model._set("A2", "=DATE(2022,11,15)");
    model._set("A3", "97");
    model._set("A4", "100");

    model._set("B1", "=DISC(A1,A2,A3,A4)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "0.037241379");
}

#[test]
fn fn_received() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2023,6,30)");
    model._set("A3", "20000");
    model._set("A4", "5%");
    model._set("A5", "3");

    model._set("B1", "=RECEIVED(A1,A2,A3,A4,A5)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "24236.387782205");
}

#[test]
fn fn_intrate() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2023,6,30)");
    model._set("A3", "10000");
    model._set("A4", "12000");
    model._set("A5", "3");

    model._set("B1", "=INTRATE(A1,A2,A3,A4,A5)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "0.057210031");
}

#[test]
fn fn_bond_functions_arguments() {
    let mut model = new_empty_model();

    // PRICEDISC: 4-5 args
    model._set("A1", "=PRICEDISC()");
    model._set("A2", "=PRICEDISC(1,2,3)");
    model._set("A3", "=PRICEDISC(1,2,3,4,5,6)");

    // PRICEMAT: 5-6 args
    model._set("B1", "=PRICEMAT()");
    model._set("B2", "=PRICEMAT(1,2,3,4)");
    model._set("B3", "=PRICEMAT(1,2,3,4,5,6,7)");

    // YIELDDISC: 4-5 args
    model._set("C1", "=YIELDDISC()");
    model._set("C2", "=YIELDDISC(1,2,3)");
    model._set("C3", "=YIELDDISC(1,2,3,4,5,6)");

    // YIELDMAT: 5-6 args
    model._set("D1", "=YIELDMAT()");
    model._set("D2", "=YIELDMAT(1,2,3,4)");
    model._set("D3", "=YIELDMAT(1,2,3,4,5,6,7)");

    // DISC: 4-5 args
    model._set("E1", "=DISC()");
    model._set("E2", "=DISC(1,2,3)");
    model._set("E3", "=DISC(1,2,3,4,5,6)");

    // RECEIVED: 4-5 args
    model._set("F1", "=RECEIVED()");
    model._set("F2", "=RECEIVED(1,2,3)");
    model._set("F3", "=RECEIVED(1,2,3,4,5,6)");

    // INTRATE: 4-5 args
    model._set("G1", "=INTRATE()");
    model._set("G2", "=INTRATE(1,2,3)");
    model._set("G3", "=INTRATE(1,2,3,4,5,6)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
    assert_eq!(model._get_text("C3"), *"#ERROR!");

    assert_eq!(model._get_text("D1"), *"#ERROR!");
    assert_eq!(model._get_text("D2"), *"#ERROR!");
    assert_eq!(model._get_text("D3"), *"#ERROR!");

    assert_eq!(model._get_text("E1"), *"#ERROR!");
    assert_eq!(model._get_text("E2"), *"#ERROR!");
    assert_eq!(model._get_text("E3"), *"#ERROR!");

    assert_eq!(model._get_text("F1"), *"#ERROR!");
    assert_eq!(model._get_text("F2"), *"#ERROR!");
    assert_eq!(model._get_text("F3"), *"#ERROR!");

    assert_eq!(model._get_text("G1"), *"#ERROR!");
    assert_eq!(model._get_text("G2"), *"#ERROR!");
    assert_eq!(model._get_text("G3"), *"#ERROR!");
}

#[test]
fn fn_bond_functions_date_boundaries() {
    let mut model = new_empty_model();

    // Date boundary values
    model._set("A1", "0"); // Below MINIMUM_DATE_SERIAL_NUMBER
    model._set("A2", "1"); // MINIMUM_DATE_SERIAL_NUMBER
    model._set("A3", "2958465"); // MAXIMUM_DATE_SERIAL_NUMBER
    model._set("A4", "2958466"); // Above MAXIMUM_DATE_SERIAL_NUMBER

    // Test settlement < minimum
    model._set("B1", "=PRICEDISC(A1,A2,0.05,100)");
    model._set("B2", "=YIELDDISC(A1,A2,95,100)");
    model._set("B3", "=DISC(A1,A2,95,100)");
    model._set("B4", "=RECEIVED(A1,A2,1000,0.05)");
    model._set("B5", "=INTRATE(A1,A2,1000,1050)");

    // Test maturity > maximum
    model._set("C1", "=PRICEDISC(A2,A4,0.05,100)");
    model._set("C2", "=YIELDDISC(A2,A4,95,100)");
    model._set("C3", "=DISC(A2,A4,95,100)");
    model._set("C4", "=RECEIVED(A2,A4,1000,0.05)");
    model._set("C5", "=INTRATE(A2,A4,1000,1050)");

    // Test PRICEMAT/YIELDMAT with issue < minimum
    model._set("D1", "=PRICEMAT(A2,A3,A1,0.06,0.05)");
    model._set("D2", "=YIELDMAT(A2,A3,A1,0.06,99)");

    // Test PRICEMAT/YIELDMAT with issue > maximum
    model._set("E1", "=PRICEMAT(A2,A3,A4,0.06,0.05)");
    model._set("E2", "=YIELDMAT(A2,A3,A4,0.06,99)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
    assert_eq!(model._get_text("B3"), *"#NUM!");
}

#[test]
fn fn_yield_invalid_frequency() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=YIELD(A1,A2,5%,99,100,0)");
    model._set("B2", "=YIELD(A1,A2,5%,99,100,3)");
    model._set("B3", "=YIELD(A1,A2,5%,99,100,5)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
    assert_eq!(model._get_text("B3"), *"#NUM!");
}

#[test]
fn fn_bond_functions_date_ordering() {
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2022,1,1)"); // settlement
    model._set("A2", "=DATE(2021,12,31)"); // maturity (before settlement)
    model._set("A3", "=DATE(2020,1,1)"); // issue

    // Test settlement >= maturity
    model._set("B1", "=PRICEDISC(A1,A2,0.05,100)");
    model._set("B2", "=YIELDDISC(A1,A2,95,100)");
    model._set("B3", "=DISC(A1,A2,95,100)");
    model._set("B4", "=RECEIVED(A1,A2,1000,0.05)");
    model._set("B5", "=INTRATE(A1,A2,1000,1050)");
    model._set("B6", "=PRICEMAT(A1,A2,A3,0.06,0.05)");
    model._set("B7", "=YIELDMAT(A1,A2,A3,0.06,99)");

    // Test settlement < issue for YIELDMAT/PRICEMAT
    model._set("A4", "=DATE(2023,1,1)"); // later issue date
    model._set("C1", "=PRICEMAT(A1,A2,A4,0.06,0.05)"); // settlement < issue
    model._set("C2", "=YIELDMAT(A1,A2,A4,0.06,99)"); // settlement < issue

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
    assert_eq!(model._get_text("B3"), *"#NUM!");
}

#[test]
fn fn_price_invalid_dates() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=PRICE(A2,A1,5%,6%,100,2)");
    model._set("B2", "=PRICE(A1,A1,5%,6%,100,2)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
}

#[test]
fn fn_bond_functions_parameter_validation() {
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2022,1,1)");
    model._set("A2", "=DATE(2022,12,31)");
    model._set("A3", "=DATE(2021,1,1)");

    // Test negative/zero prices and redemptions
    model._set("B1", "=PRICEDISC(A1,A2,0.05,0)"); // zero redemption
    model._set("B2", "=PRICEDISC(A1,A2,0,100)"); // zero discount
    model._set("B3", "=PRICEDISC(A1,A2,-0.05,100)"); // negative discount

    model._set("C1", "=YIELDDISC(A1,A2,0,100)"); // zero price
    model._set("C2", "=YIELDDISC(A1,A2,95,0)"); // zero redemption
    model._set("C3", "=YIELDDISC(A1,A2,-95,100)"); // negative price

    model._set("D1", "=DISC(A1,A2,0,100)"); // zero price
    model._set("D2", "=DISC(A1,A2,95,0)"); // zero redemption
    model._set("D3", "=DISC(A1,A2,-95,100)"); // negative price

    model._set("E1", "=RECEIVED(A1,A2,0,0.05)"); // zero investment
    model._set("E2", "=RECEIVED(A1,A2,1000,0)"); // zero discount
    model._set("E3", "=RECEIVED(A1,A2,-1000,0.05)"); // negative investment

    model._set("F1", "=INTRATE(A1,A2,0,1050)"); // zero investment
    model._set("F2", "=INTRATE(A1,A2,1000,0)"); // zero redemption
    model._set("F3", "=INTRATE(A1,A2,-1000,1050)"); // negative investment

    model._set("G1", "=PRICEMAT(A1,A2,A3,-0.06,0.05)"); // negative rate
    model._set("G2", "=PRICEMAT(A1,A2,A3,0.06,-0.05)"); // negative yield

    model._set("H1", "=YIELDMAT(A1,A2,A3,0.06,0)"); // zero price
    model._set("H2", "=YIELDMAT(A1,A2,A3,-0.06,99)"); // negative rate

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
}

#[test]
fn fn_yield_invalid_dates() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=YIELD(A2,A1,5%,99,100,2)");
    model._set("B2", "=YIELD(A1,A1,5%,99,100,2)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
}

#[test]
fn fn_price_with_basis() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=PRICE(A1,A2,5%,6%,100,2,0)");
    model._set("B2", "=PRICE(A1,A2,5%,6%,100,2,1)");

    model.evaluate();

    assert!(model._get_text("B1").parse::<f64>().is_ok());
    assert!(model._get_text("B2").parse::<f64>().is_ok());
}

#[test]
fn fn_yield_with_basis() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2023,1,1)");
    model._set("A2", "=DATE(2024,1,1)");

    model._set("B1", "=YIELD(A1,A2,5%,99,100,2,0)");
    model._set("B2", "=YIELD(A1,A2,5%,99,100,2,1)");

    model.evaluate();

    assert!(model._get_text("B1").parse::<f64>().is_ok());
    assert!(model._get_text("B2").parse::<f64>().is_ok());
}

#[test]
fn fn_price_yield_inverse_functions() {
    // Verifies PRICE and YIELD are mathematical inverses
    // Regression test for periods calculation type mismatch
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2023,1,15)");
    model._set("A2", "=DATE(2024,7,15)"); // ~1.5 years, fractional periods
    model._set("A3", "4.75%"); // coupon
    model._set("A4", "5.125%"); // yield

    model._set("B1", "=PRICE(A1,A2,A3,A4,100,2)");
    model._set("B2", "=YIELD(A1,A2,A3,B1,100,2)");

    model.evaluate();

    let calculated_yield: f64 = model._get_text("B2").parse().unwrap();
    let expected_yield = 0.05125;

    assert!(
        (calculated_yield - expected_yield).abs() < 1e-12,
        "YIELD should recover original yield: expected {expected_yield}, got {calculated_yield}"
    );
}

#[test]
fn fn_price_yield_round_trip_stability() {
    // Tests numerical stability through multiple PRICE->YIELD->PRICE cycles
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2023,3,10)");
    model._set("A2", "=DATE(2024,11,22)"); // Irregular period length
    model._set("A3", "3.25%"); // coupon rate
    model._set("A4", "4.875%"); // initial yield

    // First round-trip
    model._set("B1", "=PRICE(A1,A2,A3,A4,100,4)");
    model._set("B2", "=YIELD(A1,A2,A3,B1,100,4)");

    // Second round-trip
    model._set("B3", "=PRICE(A1,A2,A3,B2,100,4)");

    model.evaluate();

    let price1: f64 = model._get_text("B1").parse().unwrap();
    let price2: f64 = model._get_text("B3").parse().unwrap();

    assert!(
        (price1 - price2).abs() < 1e-10,
        "Round-trip should be stable: {price1} vs {price2}"
    );
}

#[test]
fn fn_bond_functions_basis_validation() {
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2022,1,1)");
    model._set("A2", "=DATE(2022,12,31)");
    model._set("A3", "=DATE(2021,1,1)");

    // Test valid basis values (0-4)
    model._set("B1", "=PRICEDISC(A1,A2,0.05,100,0)");
    model._set("B2", "=PRICEDISC(A1,A2,0.05,100,1)");
    model._set("B3", "=PRICEDISC(A1,A2,0.05,100,2)");
    model._set("B4", "=PRICEDISC(A1,A2,0.05,100,3)");
    model._set("B5", "=PRICEDISC(A1,A2,0.05,100,4)");

    // Test invalid basis values
    model._set("C1", "=PRICEDISC(A1,A2,0.05,100,-1)");
    model._set("C2", "=PRICEDISC(A1,A2,0.05,100,5)");
    model._set("C3", "=YIELDDISC(A1,A2,95,100,10)");
    model._set("C4", "=DISC(A1,A2,95,100,-5)");
    model._set("C5", "=RECEIVED(A1,A2,1000,0.05,99)");
    model._set("C6", "=INTRATE(A1,A2,1000,1050,-2)");
    model._set("C7", "=PRICEMAT(A1,A2,A3,0.06,0.05,7)");
    model._set("C8", "=YIELDMAT(A1,A2,A3,0.06,99,-3)");

    model.evaluate();

    // Valid basis should work
    assert_ne!(model._get_text("B1"), *"#ERROR!");
    assert_ne!(model._get_text("B2"), *"#ERROR!");
    assert_ne!(model._get_text("B3"), *"#ERROR!");
    assert_ne!(model._get_text("B4"), *"#ERROR!");
    assert_ne!(model._get_text("B5"), *"#ERROR!");

    // Invalid basis should error
    assert_eq!(model._get_text("C1"), *"#NUM!");
    assert_eq!(model._get_text("C2"), *"#NUM!");
    assert_eq!(model._get_text("C3"), *"#NUM!");
    assert_eq!(model._get_text("C4"), *"#NUM!");
    assert_eq!(model._get_text("C5"), *"#NUM!");
    assert_eq!(model._get_text("C6"), *"#NUM!");
    assert_eq!(model._get_text("C7"), *"#NUM!");
    assert_eq!(model._get_text("C8"), *"#NUM!");
}

#[test]
fn fn_bond_functions_relationships() {
    // Test mathematical relationships between functions
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2021,1,1)");
    model._set("A2", "=DATE(2021,7,1)");

    model._set("B1", "=PRICEDISC(A1,A2,5%,100)");
    model._set("B2", "=YIELDDISC(A1,A2,B1,100)");
    model._set("B3", "=DISC(A1,A2,B1,100)");
    model._set("B4", "=RECEIVED(A1,A2,1000,5%)");
    model._set("B5", "=INTRATE(A1,A2,1000,1050)");
    model._set("B6", "=PRICEMAT(A1,A2,DATE(2020,7,1),6%,5%)");
    model._set("B7", "=YIELDMAT(A1,A2,DATE(2020,7,1),6%,99)");

    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(97.5))
    );
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B2") {
        assert!((v - 0.051282051).abs() < 1e-6);
    }
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B3") {
        assert!((v - 0.05).abs() < 1e-6);
    }
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B4") {
        assert!((v - 1025.641025).abs() < 1e-6);
    }
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B5") {
        assert!((v - 0.10).abs() < 1e-6);
    }
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B6") {
        assert!((v - 100.414634).abs() < 1e-6);
    }
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B7") {
        assert!((v - 0.078431372).abs() < 1e-6);
    }
}
