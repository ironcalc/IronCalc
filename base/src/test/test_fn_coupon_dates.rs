#![allow(clippy::panic)]

// Coupon schedules for bonds maturing late in the month. Every coupon date is
// maturity moved back a whole number of periods (with the end-of-month rule when
// maturity is the last day of its month), so a bond maturing on the 30th keeps
// paying on the 30th after February instead of drifting to the 28th.
// Expected values are from Excel 16.

use crate::cell::CellValue;
use crate::test::util::new_empty_model;
use crate::Model;

fn number(model: &Model, cell: &str) -> f64 {
    match model.get_cell_value_by_ref(cell) {
        Ok(CellValue::Number(n)) => n,
        other => panic!("{cell} is not a number: {other:?}"),
    }
}

// Evaluates each formula in its own cell and compares it with Excel's result.
fn assert_formulas(cases: &[(&str, f64)], tolerance: f64) {
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        let value = number(&model, &format!("Sheet1!A{}", row + 1));
        assert!(
            (value - expected).abs() <= tolerance,
            "{formula}: got {value}, Excel gives {expected}"
        );
    }
}

#[test]
fn fn_coupon_dates_keep_maturity_day() {
    assert_formulas(
        &[
            // Semiannual, maturity on the 30th
            ("=COUPPCD(DATE(2024,1,15),DATE(2026,8,30),2,0)", 45168.0), // 2023-08-30
            ("=COUPNCD(DATE(2024,1,15),DATE(2026,8,30),2,0)", 45351.0), // 2024-02-29
            ("=COUPNUM(DATE(2024,1,15),DATE(2026,8,30),2,0)", 6.0),
            // Quarterly, maturity on the 30th
            ("=COUPPCD(DATE(2024,1,10),DATE(2026,5,30),4,0)", 45260.0), // 2023-11-30
            ("=COUPNCD(DATE(2024,1,10),DATE(2026,5,30),4,0)", 45351.0), // 2024-02-29
            ("=COUPNUM(DATE(2024,1,10),DATE(2026,5,30),4,0)", 10.0),
            // Semiannual and quarterly, maturity on the 29th
            ("=COUPPCD(DATE(2025,1,10),DATE(2027,8,29),2,0)", 45533.0), // 2024-08-29
            ("=COUPNCD(DATE(2025,1,10),DATE(2027,8,29),2,0)", 45716.0), // 2025-02-28
            ("=COUPPCD(DATE(2025,1,10),DATE(2027,8,29),4,0)", 45625.0), // 2024-11-29
            ("=COUPNCD(DATE(2025,1,10),DATE(2027,8,29),4,0)", 45716.0), // 2025-02-28
            // Maturity on the 31st and at the end of February: month-end coupons
            ("=COUPPCD(DATE(2024,4,1),DATE(2026,8,31),2,1)", 45351.0), // 2024-02-29
            ("=COUPNCD(DATE(2024,4,1),DATE(2026,8,31),2,1)", 45535.0), // 2024-08-31
            ("=COUPPCD(DATE(2026,8,27),DATE(2027,2,28),2,0)", 46081.0), // 2026-02-28
            ("=COUPNCD(DATE(2026,8,27),DATE(2027,2,28),2,0)", 46265.0), // 2026-08-31
            // Annual, maturity on February 29
            ("=COUPPCD(DATE(2025,6,1),DATE(2028,2,29),1,0)", 45716.0), // 2025-02-28
            ("=COUPNCD(DATE(2027,6,1),DATE(2028,2,29),1,0)", 46812.0), // 2028-02-29
        ],
        0.0,
    );
}

#[test]
fn fn_coupon_dates_settlement_day_before_coupon() {
    // Settlement on 2024-08-29 is the day before the 2024-08-30 coupon
    assert_formulas(
        &[
            ("=COUPPCD(DATE(2024,8,29),DATE(2026,8,30),2,0)", 45351.0), // 2024-02-29
            ("=COUPNCD(DATE(2024,8,29),DATE(2026,8,30),2,0)", 45534.0), // 2024-08-30
            ("=COUPNUM(DATE(2024,8,29),DATE(2026,8,30),2,0)", 5.0),
            ("=COUPDAYBS(DATE(2024,8,29),DATE(2026,8,30),2,0)", 179.0),
            ("=COUPDAYSNC(DATE(2024,8,29),DATE(2026,8,30),2,0)", 1.0),
            ("=COUPDAYBS(DATE(2024,8,29),DATE(2026,8,30),2,1)", 182.0),
            ("=COUPDAYSNC(DATE(2024,8,29),DATE(2026,8,30),2,1)", 1.0),
        ],
        0.0,
    );
}

#[test]
fn fn_coupon_day_counts_month_end() {
    assert_formulas(
        &[
            // Semiannual, maturity on the 30th
            ("=COUPDAYBS(DATE(2024,1,15),DATE(2026,8,30),2,0)", 135.0),
            ("=COUPDAYBS(DATE(2024,1,15),DATE(2026,8,30),2,1)", 138.0),
            ("=COUPDAYBS(DATE(2024,1,15),DATE(2026,8,30),2,2)", 138.0),
            ("=COUPDAYBS(DATE(2024,1,15),DATE(2026,8,30),2,3)", 138.0),
            ("=COUPDAYBS(DATE(2024,1,15),DATE(2026,8,30),2,4)", 135.0),
            ("=COUPDAYSNC(DATE(2024,1,15),DATE(2026,8,30),2,0)", 45.0),
            ("=COUPDAYSNC(DATE(2024,1,15),DATE(2026,8,30),2,1)", 45.0),
            ("=COUPDAYSNC(DATE(2024,1,15),DATE(2026,8,30),2,2)", 45.0),
            ("=COUPDAYSNC(DATE(2024,1,15),DATE(2026,8,30),2,3)", 45.0),
            ("=COUPDAYSNC(DATE(2024,1,15),DATE(2026,8,30),2,4)", 44.0),
            // Quarterly, maturity on the 30th
            ("=COUPDAYBS(DATE(2024,1,10),DATE(2026,5,30),4,0)", 40.0),
            ("=COUPDAYBS(DATE(2024,1,10),DATE(2026,5,30),4,1)", 41.0),
            ("=COUPDAYBS(DATE(2024,1,10),DATE(2026,5,30),4,4)", 40.0),
            ("=COUPDAYSNC(DATE(2024,1,10),DATE(2026,5,30),4,0)", 50.0),
            ("=COUPDAYSNC(DATE(2024,1,10),DATE(2026,5,30),4,1)", 50.0),
            ("=COUPDAYSNC(DATE(2024,1,10),DATE(2026,5,30),4,4)", 49.0),
            // Semiannual, maturity on the 29th
            ("=COUPDAYBS(DATE(2025,1,10),DATE(2027,8,29),2,0)", 131.0),
            ("=COUPDAYBS(DATE(2025,1,10),DATE(2027,8,29),2,1)", 134.0),
        ],
        0.0,
    );
}

#[test]
fn fn_coupdays_actual_actual() {
    // With basis 1 Excel's COUPDAYS is not always COUPNCD - COUPPCD
    assert_formulas(
        &[
            ("=COUPDAYS(DATE(2024,1,15),DATE(2026,8,30),2,1)", 184.0),
            ("=COUPDAYS(DATE(2025,1,10),DATE(2027,8,29),2,1)", 181.0),
            ("=COUPDAYS(DATE(2024,1,15),DATE(2026,8,31),2,1)", 184.0),
            ("=COUPDAYS(DATE(2024,4,1),DATE(2026,8,31),2,1)", 182.0),
            (
                "=COUPNCD(DATE(2024,4,1),DATE(2026,8,31),2,1)-COUPPCD(DATE(2024,4,1),DATE(2026,8,31),2,1)",
                184.0,
            ),
            ("=COUPDAYS(DATE(2026,8,27),DATE(2027,2,28),2,1)", 181.0),
            ("=COUPDAYS(DATE(2024,1,10),DATE(2026,5,31),4,1)", 92.0),
            ("=COUPDAYS(DATE(2026,4,29),DATE(2027,1,30),4,1)", 89.0),
            ("=COUPDAYS(DATE(2026,5,28),DATE(2027,5,29),4,1)", 92.0),
            ("=COUPDAYS(DATE(2025,6,1),DATE(2028,2,29),1,1)", 365.0),
            ("=COUPDAYS(DATE(2027,6,1),DATE(2028,2,29),1,1)", 366.0),
        ],
        0.0,
    );
}

#[test]
fn fn_price_yield_duration_month_end() {
    assert_formulas(
        &[
            // Semiannual, maturity on the 30th
            (
                "=PRICE(DATE(2024,1,15),DATE(2026,8,30),0.05,0.06,100,2,0)",
                97.5973528255953,
            ),
            (
                "=YIELD(DATE(2024,1,15),DATE(2026,8,30),0.05,97,100,2,0)",
                0.06253937810734618,
            ),
            (
                "=DURATION(DATE(2024,1,15),DATE(2026,8,30),0.05,0.06,2,0)",
                2.445002592587616,
            ),
            (
                "=MDURATION(DATE(2024,1,15),DATE(2026,8,30),0.05,0.06,2,0)",
                2.3737889248423456,
            ),
            (
                "=PRICE(DATE(2024,1,15),DATE(2026,8,30),0.05,0.06,100,2,1)",
                97.59915799653675,
            ),
            (
                "=YIELD(DATE(2024,1,15),DATE(2026,8,30),0.05,97,100,2,1)",
                0.06254890972663979,
            ),
            (
                "=DURATION(DATE(2024,1,15),DATE(2026,8,30),0.05,0.06,2,1)",
                2.442953412259747,
            ),
            (
                "=MDURATION(DATE(2024,1,15),DATE(2026,8,30),0.05,0.06,2,1)",
                2.371799429378395,
            ),
            // Quarterly, maturity on the 30th
            (
                "=PRICE(DATE(2024,1,10),DATE(2026,5,30),0.05,0.06,100,4,0)",
                97.78750183741766,
            ),
            (
                "=YIELD(DATE(2024,1,10),DATE(2026,5,30),0.05,97,100,4,0)",
                0.06362503975036993,
            ),
            (
                "=DURATION(DATE(2024,1,10),DATE(2026,5,30),0.05,0.06,4,0)",
                2.2525192660094557,
            ),
            (
                "=MDURATION(DATE(2024,1,10),DATE(2026,5,30),0.05,0.06,4,0)",
                2.219230803950203,
            ),
            (
                "=PRICE(DATE(2024,1,10),DATE(2026,5,30),0.05,0.06,100,4,1)",
                97.78880988515205,
            ),
            (
                "=YIELD(DATE(2024,1,10),DATE(2026,5,30),0.05,97,100,4,1)",
                0.06363322104940268,
            ),
            (
                "=DURATION(DATE(2024,1,10),DATE(2026,5,30),0.05,0.06,4,1)",
                2.250993014483204,
            ),
            (
                "=MDURATION(DATE(2024,1,10),DATE(2026,5,30),0.05,0.06,4,1)",
                2.217727107865226,
            ),
            // Settlement the day before a coupon date
            (
                "=DURATION(DATE(2024,8,29),DATE(2026,8,30),0.05,0.06,2,0)",
                1.8821396763820408,
            ),
            // Basis 1 uses the 184 day coupon period, not COUPDAYS (182)
            (
                "=PRICE(DATE(2024,4,1),DATE(2026,8,31),0.05,0.06,100,2,1)",
                97.77895184075945,
            ),
        ],
        1e-9,
    );
}
