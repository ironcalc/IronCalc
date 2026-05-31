use chrono::{Datelike, Months, NaiveDate};

use crate::{
    calc_result::CalcResult,
    constants::EXCEL_DATE_BASE,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    model::Model,
};

// ============================================================
// Date helpers
// ============================================================

fn naivedate_to_serial(d: NaiveDate) -> i64 {
    (d.num_days_from_ce() - EXCEL_DATE_BASE) as i64
}

fn last_day_of_feb(year: i32) -> u32 {
    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        29
    } else {
        28
    }
}

// Days d1→d2 via the US 30/360 convention (basis 0).
fn days_30_360_us(d1: NaiveDate, d2: NaiveDate) -> f64 {
    let y1 = d1.year();
    let m1 = d1.month() as i32;
    let mut day1 = d1.day() as i32;
    let y2 = d2.year();
    let m2 = d2.month() as i32;
    let mut day2 = d2.day() as i32;

    if (m1 == 2 && d1.day() == last_day_of_feb(y1)) || day1 == 31 {
        day1 = 30;
    }
    if m2 == 2 && d2.day() == last_day_of_feb(y2) && day1 == 30 {
        day2 = 30;
    }
    if day2 == 31 && day1 >= 30 {
        day2 = 30;
    }
    (360 * (y2 - y1) + 30 * (m2 - m1) + (day2 - day1)) as f64
}

// Days d1→d2 via the EU 30/360 convention (basis 4).
fn days_30_360_eu(d1: NaiveDate, d2: NaiveDate) -> f64 {
    let y1 = d1.year();
    let m1 = d1.month() as i32;
    let day1 = (d1.day() as i32).min(30);
    let y2 = d2.year();
    let m2 = d2.month() as i32;
    let day2 = (d2.day() as i32).min(30);
    (360 * (y2 - y1) + 30 * (m2 - m1) + (day2 - day1)) as f64
}

fn actual_days(d1: NaiveDate, d2: NaiveDate) -> f64 {
    (d2 - d1).num_days() as f64
}

// Basis-aware days from d1 to d2.
fn basis_days(d1: NaiveDate, d2: NaiveDate, basis: u32) -> f64 {
    match basis {
        0 => days_30_360_us(d1, d2),
        1 | 2 | 3 => actual_days(d1, d2),
        4 => days_30_360_eu(d1, d2),
        _ => 0.0,
    }
}

// ============================================================
// Coupon-schedule helpers
// ============================================================

// Latest coupon date that is <= settlement, anchored by maturity & frequency.
fn prev_coupon_date(settlement: i64, maturity: i64, frequency: u32) -> Result<i64, String> {
    let months_per = 12 / frequency;
    let mat = from_excel_date(maturity)?;
    let set = from_excel_date(settlement)?;

    let mat_months = mat.year() * 12 + mat.month() as i32;
    let set_months = set.year() * 12 + set.month() as i32;
    let diff_months = mat_months - set_months;

    // Estimate how many periods to subtract from maturity.
    let periods_back = ((diff_months as f64 / months_per as f64).ceil() as u32).max(1);

    let mut candidate = mat
        .checked_sub_months(Months::new(periods_back * months_per))
        .ok_or("Date arithmetic error")?;

    // Slide forward while the next coupon date is still <= settlement.
    loop {
        let next = candidate
            .checked_add_months(Months::new(months_per))
            .ok_or("Date arithmetic error")?;
        if naivedate_to_serial(next) > settlement {
            break;
        }
        candidate = next;
    }

    // Slide backward if we somehow overshot.
    while naivedate_to_serial(candidate) > settlement {
        candidate = candidate
            .checked_sub_months(Months::new(months_per))
            .ok_or("Date arithmetic error")?;
    }

    Ok(naivedate_to_serial(candidate))
}

// Earliest coupon date strictly after settlement.
fn next_coupon_date(settlement: i64, maturity: i64, frequency: u32) -> Result<i64, String> {
    let months_per = 12 / frequency;
    let prev = from_excel_date(prev_coupon_date(settlement, maturity, frequency)?)?;
    let next = prev
        .checked_add_months(Months::new(months_per))
        .ok_or("Date arithmetic error")?;
    Ok(naivedate_to_serial(next))
}

// Number of coupon payments strictly after settlement up to and including maturity.
fn coupon_count(settlement: i64, maturity: i64, frequency: u32) -> Result<u32, String> {
    let months_per = 12 / frequency as i32;
    let next = from_excel_date(next_coupon_date(settlement, maturity, frequency)?)?;
    let mat = from_excel_date(maturity)?;

    let next_months = next.year() * 12 + next.month() as i32;
    let mat_months = mat.year() * 12 + mat.month() as i32;
    let diff = mat_months - next_months;
    if diff < 0 {
        return Err("Maturity before next coupon".to_string());
    }
    Ok((diff / months_per) as u32 + 1)
}

// Total days in the coupon period that contains settlement (E).
fn coupon_days_e(
    settlement: i64,
    maturity: i64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    match basis {
        0 | 2 | 4 => Ok(360.0 / frequency as f64),
        1 => {
            let prev = from_excel_date(prev_coupon_date(settlement, maturity, frequency)?)?;
            let next = from_excel_date(next_coupon_date(settlement, maturity, frequency)?)?;
            Ok(actual_days(prev, next))
        }
        3 => Ok(365.0 / frequency as f64),
        _ => Err("Invalid basis".to_string()),
    }
}

// Accrued days from the previous coupon to settlement (A).
fn coupon_days_a(
    settlement: i64,
    maturity: i64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let prev = from_excel_date(prev_coupon_date(settlement, maturity, frequency)?)?;
    let set = from_excel_date(settlement)?;
    Ok(basis_days(prev, set, basis))
}

// ============================================================
// Validation helpers
// ============================================================

fn validate_frequency(frequency: u32, cell: CellReferenceIndex) -> Result<(), CalcResult> {
    if frequency == 1 || frequency == 2 || frequency == 4 {
        Ok(())
    } else {
        Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "frequency must be 1, 2, or 4".to_string(),
        ))
    }
}

fn validate_basis(basis: u32, cell: CellReferenceIndex) -> Result<(), CalcResult> {
    if basis <= 4 {
        Ok(())
    } else {
        Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "basis must be 0–4".to_string(),
        ))
    }
}

fn map_date_err(e: String, cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, e)
}

// ============================================================
// Core bond price computation (standard coupon bond)
// ============================================================

// Returns the clean price (per `redemption` par) for a standard coupon bond.
fn compute_price(
    settlement: i64,
    maturity: i64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let n = coupon_count(settlement, maturity, frequency)? as f64;
    let e = coupon_days_e(settlement, maturity, frequency, basis)?;
    let a = coupon_days_a(settlement, maturity, frequency, basis)?;
    let dsc_over_e = (e - a) / e;

    let coupon = 100.0 * rate / frequency as f64;
    let v = 1.0 + yld / frequency as f64;
    let first_term = redemption / v.powf(dsc_over_e + n - 1.0);

    let mut second_term = 0.0;
    for k in 1..=(n as u32) {
        let t = dsc_over_e + k as f64 - 1.0;
        second_term += coupon / v.powf(t);
    }

    let last_term = coupon * (a / e);

    Ok(first_term + second_term - last_term)
}

// ============================================================
// Yield solver (Newton-Raphson + bisection)
// ============================================================

fn compute_yield_from_price(
    target: f64,
    settlement: i64,
    maturity: i64,
    rate: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let obj = |y: f64| -> Option<f64> {
        compute_price(settlement, maturity, rate, y, redemption, frequency, basis)
            .ok()
            .map(|p| p - target)
    };

    let h = 1e-5;
    let eps = 1e-8;

    // Newton-Raphson starting near the coupon rate.
    let mut y = rate;
    for _ in 0..100 {
        let fv = match obj(y) {
            Some(v) if v.is_finite() => v,
            _ => break,
        };
        let fp = match (obj(y + h), obj(y - h)) {
            (Some(a), Some(b)) => (a - b) / (2.0 * h),
            _ => break,
        };
        if !fp.is_finite() || fp.abs() < 1e-15 {
            break;
        }
        let yn = y - fv / fp;
        if yn <= -1.0 {
            break;
        }
        if (yn - y).abs() < eps {
            return Ok(yn);
        }
        y = yn;
    }

    // Bisection fallback.
    let x1 = -0.9999_f64;
    let x2 = 100.0_f64;
    let f1 = obj(x1).ok_or("bisection failed")?;
    let f2 = obj(x2).ok_or("bisection failed")?;
    if f1 * f2 > 0.0 {
        return Err("YIELD: failed to converge".to_string());
    }
    let (mut lo, mut hi, fl_sign) = if f1 < 0.0 {
        (x1, x2, -1.0_f64)
    } else {
        (x2, x1, 1.0_f64)
    };
    let _ = fl_sign; // used only implicitly through lo/hi ordering

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = obj(mid).ok_or("bisection failed")?;
        if fm.abs() < eps || (hi - lo).abs() < 1e-12 {
            return Ok(mid);
        }
        let f_lo = obj(lo).ok_or("bisection failed")?;
        if f_lo * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Err("YIELD: failed to converge".to_string())
}

// ============================================================
// Odd-first-period price
// ============================================================

// Clean price for a bond with an odd first coupon period.
// settlement:   settlement date
// maturity:     maturity date
// issue:        issue date (defines start of odd first period)
// first_coupon: first coupon date
// rate, yld, redemption, frequency, basis: standard
fn compute_oddfprice(
    settlement: i64,
    maturity: i64,
    issue: i64,
    first_coupon: i64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    // If settlement is at or after first_coupon, use the standard price formula.
    if settlement >= first_coupon {
        return compute_price(
            settlement, maturity, rate, yld, redemption, frequency, basis,
        );
    }

    let months_per = 12 / frequency;
    let fc_date = from_excel_date(first_coupon)?;
    let mat_date = from_excel_date(maturity)?;

    // Quasi-coupon period (E): the period [prev_fc, first_coupon].
    let prev_fc = fc_date
        .checked_sub_months(Months::new(months_per))
        .ok_or("Date arithmetic error")?;
    let e: f64 = match basis {
        0 | 2 | 4 => 360.0 / frequency as f64,
        1 => actual_days(prev_fc, fc_date),
        3 => 365.0 / frequency as f64,
        _ => return Err("Invalid basis".to_string()),
    };

    let issue_date = from_excel_date(issue)?;
    let set_date = from_excel_date(settlement)?;
    let nc = basis_days(issue_date, fc_date, basis);

    // DSC/E: time from settlement to first coupon, in coupon periods. Accumulates
    // 1.0 per full period + the fractional last step, so this correctly handles
    // settlement more than one period before FC as well as the on-coupon-date case.
    let dsc_over_e = {
        let mut result = 0.0_f64;
        let mut period_end = fc_date;
        loop {
            let period_start = period_end
                .checked_sub_months(Months::new(months_per))
                .ok_or("Date arithmetic error")?;
            let e_i = match basis {
                0 | 2 | 4 => 360.0 / frequency as f64,
                1 => actual_days(period_start, period_end),
                3 => 365.0 / frequency as f64,
                _ => unreachable!(),
            };
            if set_date >= period_start {
                let dsc_i = basis_days(set_date, period_end, basis);
                result += dsc_i / e_i;
                break;
            }
            result += 1.0;
            period_end = period_start;
        }
        result
    };

    // N = total coupon payments from first_coupon to maturity (inclusive).
    let fc_months = fc_date.year() * 12 + fc_date.month() as i32;
    let mat_months = mat_date.year() * 12 + mat_date.month() as i32;
    let diff = mat_months - fc_months;
    if diff < 0 {
        return Err("first_coupon after maturity".to_string());
    }
    let n = (diff / months_per as i32) as u32 + 1;

    let coupon = 100.0 * rate / frequency as f64;
    let v = 1.0 + yld / frequency as f64;

    // For actual-day bases (1, 2) with a long first period (NC > E), adjacent
    // quasi-coupon periods can have different actual lengths (e.g. one might
    // contain Feb 29). We must weight each sub-period's days by its own actual
    // NL_i rather than the single constant E. For 30/360 bases (0, 4) every
    // period is exactly 360 days, so the simple nc/e is correct.
    let (nc_over_e, a_over_e) = if matches!(basis, 1 | 2) && nc > e {
        let mut sum_nc = 0.0_f64;
        let mut sum_a = 0.0_f64;
        let mut period_end = fc_date;
        loop {
            let period_start = period_end
                .checked_sub_months(Months::new(months_per))
                .ok_or("Date arithmetic error")?;
            let nl_i = actual_days(period_start, period_end);
            println!(
                "Period {}: {} to {}, NL_i = {}",
                n - sum_nc as u32,
                period_start,
                period_end,
                nl_i
            );

            if issue_date < period_end {
                // DC_i: odd-period days falling in [period_start, period_end]
                let sub_start = if issue_date > period_start {
                    issue_date
                } else {
                    period_start
                };
                sum_nc += actual_days(sub_start, period_end) / nl_i;
                println!(
                    "  Adding DC_i: {} (from {} to {})",
                    actual_days(sub_start, period_end),
                    sub_start,
                    period_end
                );

                // A_i: those days that are also before (or at) settlement
                if set_date > period_start {
                    let a_start = if issue_date > period_start {
                        issue_date
                    } else {
                        period_start
                    };
                    let a_end = if set_date < period_end {
                        set_date
                    } else {
                        period_end
                    };
                    sum_a += actual_days(a_start, a_end) / nl_i;
                }
            }

            if period_start <= issue_date {
                break;
            }
            period_end = period_start;
        }
        (sum_nc, sum_a)
    } else {
        let a = basis_days(issue_date, set_date, basis);
        (nc / e, a / e)
    };
    println!("NC/E: {}, A/E: {}", nc_over_e, a_over_e);

    let first_term = redemption / v.powf(dsc_over_e + (n as f64 - 1.0));

    println!("First term: {}", first_term);

    let second_term = coupon * nc_over_e / v.powf(dsc_over_e);
    println!("Second term: {}", second_term);
    let mut third_term = 0.0;
    for k in 1..n {
        let kf = k as f64;
        third_term += coupon / v.powf(dsc_over_e + kf);
    }
    println!("Third term: {}", third_term);

    let last_term = coupon * a_over_e;
    println!("Last term: {}", last_term);
    Ok(first_term + second_term + third_term - last_term)
}
// ============================================================
// Odd-last-period price
// ============================================================

// Clean price for a bond with an odd last coupon period.
// last_interest: last regular interest (coupon) date before the odd last period
fn compute_oddlprice(
    settlement: i64,
    maturity: i64,
    last_interest: i64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let li_date = from_excel_date(last_interest)?;
    let mat_date = from_excel_date(maturity)?;
    let set_date = from_excel_date(settlement)?;

    // NLi: quasi-coupon period (E) for the odd last period.
    let nli: f64 = match basis {
        0 | 2 | 4 => 360.0 / frequency as f64,
        1 => {
            // E = actual days in one regular coupon period (anchored on LI).
            let months_per = 12 / frequency;
            let next_li = li_date
                .checked_add_months(Months::new(months_per))
                .ok_or("Date arithmetic error")?;
            actual_days(li_date, next_li)
        }
        3 => 365.0 / frequency as f64,
        _ => return Err("Invalid basis".to_string()),
    };

    // NC: days in the odd last period (from last_interest to maturity).
    let nc = basis_days(li_date, mat_date, basis);

    if settlement >= last_interest {
        // --- Case 1: settlement is in the odd last period ---
        // Only one remaining payment: redemption + odd coupon at maturity.
        let ai_days = basis_days(li_date, set_date, basis); // accrued since last_interest
        let dsc = nc - ai_days; // days from settlement to maturity (basis-adjusted)
        let coupon = 100.0 * rate / frequency as f64;

        let dirty =
            (redemption + coupon * nc / nli) / (1.0 + yld / frequency as f64).powf(dsc / nli);
        let ai = coupon * ai_days / nli;
        Ok(dirty - ai)
    } else {
        // --- Case 2: settlement is before last_interest ---
        // N regular coupons remain (from next coupon after settlement to last_interest),
        // followed by the odd final payment at maturity.
        let n = coupon_count(settlement, last_interest, frequency)? as f64;
        let e = coupon_days_e(settlement, last_interest, frequency, basis)?;
        let a = coupon_days_a(settlement, last_interest, frequency, basis)?;
        let dsc_over_e = (e - a) / e;

        let coupon = 100.0 * rate / frequency as f64;
        let v = 1.0 + yld / frequency as f64;

        // Regular coupons
        let mut dirty = 0.0;
        for k in 1..=(n as u32) {
            dirty += coupon / v.powf(dsc_over_e + k as f64 - 1.0);
        }

        // Final odd payment at time (N-1+DSC/E) + NC/NLi from settlement.
        let t_final = dsc_over_e + (n - 1.0) + nc / nli;
        dirty += (redemption + coupon * nc / nli) / v.powf(t_final);

        // Accrued interest (from last regular coupon before settlement to settlement).
        let ai = coupon * (a / e);
        Ok(dirty - ai)
    }
}

// ============================================================
// Generic yield solver (wraps any price function)
// ============================================================

fn solve_yield<F>(target_price: f64, rate: f64, price_fn: F) -> Result<f64, String>
where
    F: Fn(f64) -> Option<f64>,
{
    let obj = |y: f64| price_fn(y).map(|p| p - target_price);

    let h = 1e-5;
    let eps = 1e-8;

    let mut y = rate;
    for _ in 0..100 {
        let fv = match obj(y) {
            Some(v) if v.is_finite() => v,
            _ => break,
        };
        let fp = match (obj(y + h), obj(y - h)) {
            (Some(a), Some(b)) => (a - b) / (2.0 * h),
            _ => break,
        };
        if !fp.is_finite() || fp.abs() < 1e-15 {
            break;
        }
        let yn = y - fv / fp;
        if yn <= -1.0 {
            break;
        }
        if (yn - y).abs() < eps {
            return Ok(yn);
        }
        y = yn;
    }

    // Bisection fallback
    let (x1, x2) = (-0.9999_f64, 100.0_f64);
    let f1 = obj(x1).ok_or("bisection failed")?;
    let f2 = obj(x2).ok_or("bisection failed")?;
    if f1 * f2 > 0.0 {
        return Err("yield solver: failed to converge".to_string());
    }
    let (mut lo, mut hi) = if f1 < 0.0 { (x1, x2) } else { (x2, x1) };

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = obj(mid).ok_or("bisection failed")?;
        if fm.abs() < eps || (hi - lo).abs() < 1e-12 {
            return Ok(mid);
        }
        let f_lo = obj(lo).ok_or("bisection failed")?;
        if f_lo * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Err("yield solver: failed to converge".to_string())
}

// ============================================================
// Shared argument parser for COUP* functions
// (settlement, maturity, frequency, [basis])
// ============================================================

fn parse_coupon_args(
    model: &mut Model<'_>,
    args: &[Node],
    cell: CellReferenceIndex,
) -> Result<(i64, i64, u32, u32), CalcResult> {
    let arg_count = args.len();
    if !(3..=4).contains(&arg_count) {
        return Err(CalcResult::new_args_number_error(cell));
    }
    let settlement = match model.get_number_no_bools(&args[0], cell) {
        Ok(f) => f.floor() as i64,
        Err(e) => return Err(e),
    };
    let maturity = match model.get_number_no_bools(&args[1], cell) {
        Ok(f) => f.floor() as i64,
        Err(e) => return Err(e),
    };
    let frequency = match model.get_number_no_bools(&args[2], cell) {
        Ok(f) => f.floor() as u32,
        Err(e) => return Err(e),
    };
    let basis = if arg_count == 4 {
        match model.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return Err(e),
        }
    } else {
        0
    };
    if let Err(e) = validate_frequency(frequency, cell) {
        return Err(e);
    }
    if let Err(e) = validate_basis(basis, cell) {
        return Err(e);
    }
    if settlement >= maturity {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "settlement must be before maturity".to_string(),
        ));
    }
    Ok((settlement, maturity, frequency, basis))
}

// ============================================================
// Model implementations
// ============================================================

impl<'a> Model<'a> {
    // DURATION(settlement, maturity, coupon, yld, frequency, [basis])
    pub(crate) fn fn_duration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(5..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let rate = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let yld = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        let basis = if arg_count == 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f.floor() as u32,
                Err(e) => return e,
            }
        } else {
            0
        };

        if let Err(e) = validate_frequency(frequency, cell) {
            return e;
        }
        if let Err(e) = validate_basis(basis, cell) {
            return e;
        }
        if rate < 0.0 || yld < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "rate and yld must be >= 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }

        let n = match coupon_count(settlement, maturity, frequency) {
            Ok(v) => v as f64,
            Err(e) => return map_date_err(e, cell),
        };
        let e = match coupon_days_e(settlement, maturity, frequency, basis) {
            Ok(v) => v,
            Err(e) => return map_date_err(e, cell),
        };
        let a = match coupon_days_a(settlement, maturity, frequency, basis) {
            Ok(v) => v,
            Err(e) => return map_date_err(e, cell),
        };
        let dsc_over_e = (e - a) / e;

        let redemption = 100.0;
        let coupon = redemption * rate / frequency as f64;
        let v = 1.0 + yld / frequency as f64;

        let mut num = 0.0; // sum of t * PV(CF)
        let mut den = 0.0; // dirty price (sum of PV(CF))

        for k in 1..=(n as u32) {
            let t = dsc_over_e + k as f64 - 1.0;
            let pv = coupon / v.powf(t);
            num += t * pv;
            den += pv;
        }
        let t_final = dsc_over_e + n - 1.0;
        let pv_red = redemption / v.powf(t_final);
        num += t_final * pv_red;
        den += pv_red;

        if den == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }

        CalcResult::Number(num / den / frequency as f64)
    }

    // MDURATION(settlement, maturity, coupon, yld, frequency, [basis])
    pub(crate) fn fn_mduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(5..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        // Reuse DURATION, then apply the Modified Duration adjustment.
        let yld = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        match self.fn_duration(args, cell) {
            CalcResult::Number(d) => CalcResult::Number(d / (1.0 + yld / frequency as f64)),
            other => other,
        }
    }

    // PRICE(settlement, maturity, rate, yld, redemption, frequency, [basis])
    pub(crate) fn fn_price(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(6..=7).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let rate = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let yld = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let redemption = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        let basis = if arg_count == 7 {
            match self.get_number_no_bools(&args[6], cell) {
                Ok(f) => f.floor() as u32,
                Err(e) => return e,
            }
        } else {
            0
        };

        if let Err(e) = validate_frequency(frequency, cell) {
            return e;
        }
        if let Err(e) = validate_basis(basis, cell) {
            return e;
        }
        if rate < 0.0 || yld < 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "rate and yld must be >= 0; redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }

        match compute_price(
            settlement, maturity, rate, yld, redemption, frequency, basis,
        ) {
            Ok(p) => CalcResult::Number(p),
            Err(e) => map_date_err(e, cell),
        }
    }

    // YIELD(settlement, maturity, rate, pr, redemption, frequency, [basis])
    pub(crate) fn fn_yield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(6..=7).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let rate = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let pr = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let redemption = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        let basis = if arg_count == 7 {
            match self.get_number_no_bools(&args[6], cell) {
                Ok(f) => f.floor() as u32,
                Err(e) => return e,
            }
        } else {
            0
        };

        if let Err(e) = validate_frequency(frequency, cell) {
            return e;
        }
        if let Err(e) = validate_basis(basis, cell) {
            return e;
        }
        if rate < 0.0 || pr <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "pr and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }

        match compute_yield_from_price(pr, settlement, maturity, rate, redemption, frequency, basis)
        {
            Ok(y) => CalcResult::Number(y),
            Err(e) => map_date_err(e, cell),
        }
    }

    // ODDFPRICE(settlement, maturity, issue, first_coupon, rate, yld, redemption, frequency, [basis])
    pub(crate) fn fn_oddfprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(8..=9).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let issue = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let first_coupon = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let rate = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let yld = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let redemption = match self.get_number_no_bools(&args[6], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[7], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        let basis = if arg_count == 9 {
            match self.get_number_no_bools(&args[8], cell) {
                Ok(f) => f.floor() as u32,
                Err(e) => return e,
            }
        } else {
            0
        };

        if let Err(e) = validate_frequency(frequency, cell) {
            return e;
        }
        if let Err(e) = validate_basis(basis, cell) {
            return e;
        }
        if issue >= first_coupon || first_coupon > maturity || settlement >= maturity {
            return CalcResult::new_error(Error::NUM, cell, "invalid date ordering".to_string());
        }
        if rate < 0.0 || yld < 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "invalid rate/yld/redemption".to_string(),
            );
        }

        match compute_oddfprice(
            settlement,
            maturity,
            issue,
            first_coupon,
            rate,
            yld,
            redemption,
            frequency,
            basis,
        ) {
            Ok(p) => CalcResult::Number(p),
            Err(e) => map_date_err(e, cell),
        }
    }

    // ODDFYIELD(settlement, maturity, issue, first_coupon, rate, pr, redemption, frequency, [basis])
    pub(crate) fn fn_oddfyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(8..=9).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let issue = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let first_coupon = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let rate = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let pr = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let redemption = match self.get_number_no_bools(&args[6], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[7], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        let basis = if arg_count == 9 {
            match self.get_number_no_bools(&args[8], cell) {
                Ok(f) => f.floor() as u32,
                Err(e) => return e,
            }
        } else {
            0
        };

        if let Err(e) = validate_frequency(frequency, cell) {
            return e;
        }
        if let Err(e) = validate_basis(basis, cell) {
            return e;
        }
        if issue >= first_coupon || first_coupon > maturity || settlement >= maturity {
            return CalcResult::new_error(Error::NUM, cell, "invalid date ordering".to_string());
        }
        if rate < 0.0 || pr <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "invalid rate/pr/redemption".to_string(),
            );
        }

        let price_fn = |yld: f64| {
            compute_oddfprice(
                settlement,
                maturity,
                issue,
                first_coupon,
                rate,
                yld,
                redemption,
                frequency,
                basis,
            )
            .ok()
        };

        match solve_yield(pr, rate, price_fn) {
            Ok(y) => CalcResult::Number(y),
            Err(e) => map_date_err(e, cell),
        }
    }

    // ODDLPRICE(settlement, maturity, last_interest, rate, yld, redemption, frequency, [basis])
    pub(crate) fn fn_oddlprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(7..=8).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let last_interest = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let rate = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let yld = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let redemption = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[6], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        let basis = if arg_count == 8 {
            match self.get_number_no_bools(&args[7], cell) {
                Ok(f) => f.floor() as u32,
                Err(e) => return e,
            }
        } else {
            0
        };

        if let Err(e) = validate_frequency(frequency, cell) {
            return e;
        }
        if let Err(e) = validate_basis(basis, cell) {
            return e;
        }
        if last_interest >= maturity || settlement >= maturity {
            return CalcResult::new_error(Error::NUM, cell, "invalid date ordering".to_string());
        }
        if rate < 0.0 || yld < 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "invalid rate/yld/redemption".to_string(),
            );
        }

        match compute_oddlprice(
            settlement,
            maturity,
            last_interest,
            rate,
            yld,
            redemption,
            frequency,
            basis,
        ) {
            Ok(p) => CalcResult::Number(p),
            Err(e) => map_date_err(e, cell),
        }
    }

    // ODDLYIELD(settlement, maturity, last_interest, rate, pr, redemption, frequency, [basis])
    pub(crate) fn fn_oddlyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(7..=8).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let last_interest = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(e) => return e,
        };
        let rate = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let pr = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let redemption = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let frequency = match self.get_number_no_bools(&args[6], cell) {
            Ok(f) => f.floor() as u32,
            Err(e) => return e,
        };
        let basis = if arg_count == 8 {
            match self.get_number_no_bools(&args[7], cell) {
                Ok(f) => f.floor() as u32,
                Err(e) => return e,
            }
        } else {
            0
        };

        if let Err(e) = validate_frequency(frequency, cell) {
            return e;
        }
        if let Err(e) = validate_basis(basis, cell) {
            return e;
        }
        if last_interest >= maturity || settlement >= maturity {
            return CalcResult::new_error(Error::NUM, cell, "invalid date ordering".to_string());
        }
        if rate < 0.0 || pr <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "invalid rate/pr/redemption".to_string(),
            );
        }

        let price_fn = |yld: f64| {
            compute_oddlprice(
                settlement,
                maturity,
                last_interest,
                rate,
                yld,
                redemption,
                frequency,
                basis,
            )
            .ok()
        };

        match solve_yield(pr, rate, price_fn) {
            Ok(y) => CalcResult::Number(y),
            Err(e) => map_date_err(e, cell),
        }
    }

    // COUPDAYBS(settlement, maturity, frequency, [basis])
    // Days from the beginning of the coupon period to settlement.
    pub(crate) fn fn_coupdaybs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, frequency, basis) = match parse_coupon_args(self, args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match coupon_days_a(settlement, maturity, frequency, basis) {
            Ok(v) => CalcResult::Number(v),
            Err(e) => map_date_err(e, cell),
        }
    }

    // COUPDAYS(settlement, maturity, frequency, [basis])
    // Total days in the coupon period that contains settlement.
    pub(crate) fn fn_coupdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, frequency, basis) = match parse_coupon_args(self, args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match coupon_days_e(settlement, maturity, frequency, basis) {
            Ok(v) => CalcResult::Number(v),
            Err(e) => map_date_err(e, cell),
        }
    }

    // COUPDAYSNC(settlement, maturity, frequency, [basis])
    // Days from settlement to the next coupon date.
    pub(crate) fn fn_coupdaysnc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, frequency, basis) = match parse_coupon_args(self, args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let next = match next_coupon_date(settlement, maturity, frequency) {
            Ok(v) => v,
            Err(e) => return map_date_err(e, cell),
        };
        let set_date = match from_excel_date(settlement) {
            Ok(d) => d,
            Err(e) => return map_date_err(e, cell),
        };
        let next_date = match from_excel_date(next) {
            Ok(d) => d,
            Err(e) => return map_date_err(e, cell),
        };
        CalcResult::Number(basis_days(set_date, next_date, basis))
    }

    // COUPNCD(settlement, maturity, frequency, [basis])
    // Next coupon date after settlement (returned as an Excel serial date).
    pub(crate) fn fn_coupncd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, frequency, _basis) = match parse_coupon_args(self, args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match next_coupon_date(settlement, maturity, frequency) {
            Ok(serial) => CalcResult::Number(serial as f64),
            Err(e) => map_date_err(e, cell),
        }
    }

    // COUPNUM(settlement, maturity, frequency, [basis])
    // Number of coupons payable between settlement and maturity.
    pub(crate) fn fn_coupnum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, frequency, _basis) = match parse_coupon_args(self, args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match coupon_count(settlement, maturity, frequency) {
            Ok(n) => CalcResult::Number(n as f64),
            Err(e) => map_date_err(e, cell),
        }
    }

    // COUPPCD(settlement, maturity, frequency, [basis])
    // Most recent coupon date at or before settlement (returned as an Excel serial date).
    pub(crate) fn fn_couppcd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, frequency, _basis) = match parse_coupon_args(self, args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match prev_coupon_date(settlement, maturity, frequency) {
            Ok(serial) => CalcResult::Number(serial as f64),
            Err(e) => map_date_err(e, cell),
        }
    }
}
