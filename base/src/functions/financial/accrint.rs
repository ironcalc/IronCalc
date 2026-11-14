use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
    types::DateSystem,
};

use super::calendar::{
    actual_days, add_months, day_from_ymd, day_of, is_last_day_of_month, last_day_of_month, Day,
};

// ---------------------------------------------------------------------------
// ACCRINT day-count helpers
//
// Port of the ACCRINT algorithm as implemented in the
// `ExcelFinancialFunctions` library (bonds.fs / daycountbasis.fs)
// See https://github.com/fsprojects/ExcelFinancialFunctions
// ---------------------------------------------------------------------------

// The 30/360 rules look at the Gregorian end of February
fn acc_is_last_feb(d: Day) -> bool {
    d.month == 2 && d.day == last_day_of_month(d.year, 2)
}

// EFF `changeMonth`: AddMonths (clamping the day of the month to the length of
// the target month), then optionally snap to the last day of the target month.
// `return_last_day` is fixed for the whole coupon schedule (= whether
// `first_interest` is an end-of-month date).
fn acc_change_month(
    d: Day,
    n: i32,
    return_last_day: bool,
    system: DateSystem,
) -> Result<Day, String> {
    add_months(d, n, return_last_day, system)
}

fn acc_days(after: Day, before: Day) -> f64 {
    actual_days(before, after)
}

fn acc_date_diff_360(sd: i32, sm: i32, sy: i32, ed: i32, em: i32, ey: i32) -> f64 {
    ((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64
}

// US (NASD) 30/360 day count, basis 0 and the denominator convention of the
// Actual/360 basis. `modify_both` selects Excel's `ModifyBothDates` variant.
fn acc_date_diff_360_us(start: Day, end: Day, modify_both: bool) -> f64 {
    let (mut sd, sm, sy) = (start.day as i32, start.month as i32, start.year);
    let (mut ed, em, ey) = (end.day as i32, end.month as i32, end.year);
    if acc_is_last_feb(end) && (acc_is_last_feb(start) || modify_both) {
        ed = 30;
    }
    if ed == 31 && (sd >= 30 || modify_both) {
        ed = 30;
    }
    if sd == 31 {
        sd = 30;
    }
    if acc_is_last_feb(start) {
        sd = 30;
    }
    acc_date_diff_360(sd, sm, sy, ed, em, ey)
}

// European 30/360 day count, basis 4.
fn acc_date_diff_360_eu(start: Day, end: Day) -> f64 {
    let sd = if start.day == 31 {
        30
    } else {
        start.day as i32
    };
    let ed = if end.day == 31 { 30 } else { end.day as i32 };
    acc_date_diff_360(
        sd,
        start.month as i32,
        start.year,
        ed,
        end.month as i32,
        end.year,
    )
}

// Excel's Actual/365 denominator convention (basis 3).
fn acc_date_diff_365(start: Day, end: Day, system: DateSystem) -> Result<f64, String> {
    let sd = if start.day > 28 && start.month == 2 {
        28
    } else {
        start.day
    };
    let ed = if end.day > 28 && end.month == 2 {
        28
    } else {
        end.day
    };
    let startd = day_from_ymd(start.year, start.month, sd, system)?;
    let endd = day_from_ymd(end.year, end.month, ed, system)?;
    Ok(((end.year - start.year) * 365) as f64 + acc_days(endd, startd))
}

// EFF `findPcdNcd`: step coupon boundaries from `start` by `num_months` until
// `end` is reached, returning (front, trailing) at the stopping point.
fn acc_find_pcd_ncd(
    start: Day,
    end: Day,
    num_months: i32,
    return_last: bool,
    system: DateSystem,
) -> Result<(Day, Day), String> {
    let mut front = start;
    let mut trailing = end;
    loop {
        let stop = if num_months > 0 {
            front >= end
        } else {
            front <= end
        };
        if stop {
            return Ok((front, trailing));
        }
        trailing = front;
        front = acc_change_month(front, num_months, return_last, system)?;
    }
}

// EFF `DaysBetween` for a given basis. `numerator` selects the numerator vs
// denominator convention used by the Actual/360 and Actual/365 bases.
pub(super) fn acc_days_between(
    start: Day,
    end: Day,
    numerator: bool,
    basis: i32,
    system: DateSystem,
) -> Result<f64, String> {
    let days = match basis {
        0 => acc_date_diff_360_us(start, end, false),
        4 => acc_date_diff_360_eu(start, end),
        2 => {
            if numerator {
                acc_days(end, start)
            } else {
                acc_date_diff_360_us(start, end, false)
            }
        }
        3 => {
            if numerator {
                acc_days(end, start)
            } else {
                acc_date_diff_365(start, end, system)?
            }
        }
        _ => acc_days(end, start), // Actual/Actual (basis 1)
    };
    Ok(days)
}

// EFF `CoupDays`: the normal length of a coupon period. For Actual/Actual it
// is the actual length of the period bracketing `settl`, anchored on `mat`.
fn acc_coup_days(
    settl: Day,
    mat: Day,
    freq: i32,
    basis: i32,
    system: DateSystem,
) -> Result<f64, String> {
    let coup_days = match basis {
        0 | 2 | 4 => 360.0 / freq as f64,
        3 => 365.0 / freq as f64,
        _ => {
            let num_months = -(12 / freq);
            let (pcd, ncd) = acc_find_pcd_ncd(
                mat,
                settl,
                num_months,
                is_last_day_of_month(mat, system),
                system,
            )?;
            acc_days(ncd, pcd)
        }
    };
    Ok(coup_days)
}

// ACCRINT
//
// Accrued interest = par * rate / frequency * Σ contributionᵢ, summed over the
// quasi-coupon periods spanned by the accrual interval. The schedule is anchored
// on `first_interest` and stepped by 12/frequency months. The settlement (last)
// period contributes a prorated slice; each earlier full period contributes
// `calc_method` (1 for the default issue→settlement, 0 for
// first_interest→settlement); the issue-stub period contributes its own prorated
// slice.
#[allow(clippy::too_many_arguments)]
fn compute_accrint(
    system: DateSystem,
    issue: Day,
    first_interest: Day,
    settlement: Day,
    rate: f64,
    par: f64,
    frequency: i32,
    basis: i32,
    calc_method: bool,
) -> Result<f64, String> {
    let num_months = 12 / frequency;
    let freq = frequency as f64;
    let end_month_bond = is_last_day_of_month(first_interest, system);
    let calc = if calc_method { 1.0 } else { 0.0 };

    // Regular coupon period [reg_pcd, first_interest]; its length is the
    // denominator used for the settlement slice.
    let reg_pcd = acc_change_month(first_interest, -num_months, end_month_bond, system)?;
    // Coupon date immediately before settlement. When settlement is past
    // first_interest (only meaningful for the default calc_method) walk the
    // schedule forward to find it; otherwise it is the regular coupon start.
    let pcd = if settlement > first_interest && calc_method {
        acc_find_pcd_ncd(
            first_interest,
            settlement,
            num_months,
            end_month_bond,
            system,
        )?
        .1
    } else {
        reg_pcd
    };

    let first_date = if issue > pcd { issue } else { pcd };
    let days0 = acc_days_between(first_date, settlement, true, basis, system)?;
    let coup_days0 = acc_coup_days(reg_pcd, first_interest, frequency, basis, system)?;
    let mut acc = days0 / coup_days0;

    // Walk coupon periods backward from `pcd` toward `issue`.
    let mut front = pcd;
    while front > issue {
        let period_end = front;
        front = acc_change_month(front, -num_months, end_month_bond, system)?;
        let period_start = front;
        if issue <= period_start {
            // Full coupon period inside the accrual span.
            acc += calc;
        } else {
            // The issue falls inside this period (the issue stub).
            let numerator = if basis == 0 {
                acc_date_diff_360_us(issue, period_end, false)
            } else {
                acc_days_between(issue, period_end, true, basis, system)?
            };
            // The normal length of the period. For the Actual/360 and
            // Actual/365 bases it is the fixed 360/frequency and 365/frequency
            // (Excel: ACCRINT(61,425,300,0.1,1000,2,2) divides the stub that
            // ends on 31 August 1900 by 180, not by its 30/360 length 181)
            let denominator = if basis == 0 {
                acc_date_diff_360_us(period_start, period_end, true)
            } else if basis == 2 {
                360.0 / freq
            } else if basis == 3 {
                365.0 / freq
            } else {
                acc_days_between(period_start, period_end, false, basis, system)?
            };
            acc += numerator / denominator;
        }
    }

    Ok(par * rate / freq * acc)
}

impl<'a> Model<'a> {
    // ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])
    //
    // Canonical specification: Mayle, *Standard Securities Calculation Methods*
    // (SIA / SIFMA), generalized by the Microsoft DAX `ACCRINT` documentation:
    //
    //   AI = par * (rate / frequency) * Σᵢ (Aᵢ / NLᵢ),  i = 1..NC
    //
    // where the sum runs over the NC quasi-coupon periods spanned by the
    // accrual interval. For each period i, `Aᵢ` is the number of accrued
    // days that fall inside the period under the given day-count basis,
    // and `NLᵢ` is the normal length of the period under the same basis.
    //
    // The quasi-coupon period boundaries are derived by walking backward
    // from `first_interest` (= Mayle's `CPNDT_1`, the first coupon date
    // after settlement) at intervals of 12/frequency months, snapped to
    // the end of the month when first_interest is itself an end-of-month
    // date. The walk stops once the period containing the accrual start
    // is reached.
    //
    // `calc_method` selects the accrual start:
    //   - TRUE  (default): accrue from `issue` to `settlement`
    //   - FALSE          : accrue from `first_interest` to `settlement`
    //
    // Boundary case: when `settlement` coincides with a coupon boundary
    // (issue = settlement, or accrual reduces to zero days), return 0.
    // The `issue >= settlement` case still errors per the MS spec.
    pub(crate) fn fn_accrint(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(6..=8).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let issue_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let first_interest_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let settlement_serial = match self.get_number(&args[2], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let par = match self.get_number(&args[4], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let frequency = match self.get_number(&args[5], cell) {
            Ok(f) => f.floor() as i32,
            Err(s) => return s,
        };
        let basis = if arg_count > 6 {
            match self.get_number(&args[6], cell) {
                Ok(f) => f.floor() as i32,
                Err(s) => return s,
            }
        } else {
            0
        };
        let calc_method = if arg_count > 7 {
            match self.get_boolean(&args[7], cell) {
                Ok(b) => b,
                Err(s) => return s,
            }
        } else {
            true
        };
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate must be > 0".to_string());
        }
        if par <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "par must be > 0".to_string());
        }
        if !matches!(frequency, 1 | 2 | 4) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "frequency must be 1, 2, or 4".to_string(),
            );
        }
        if !(0..=4).contains(&basis) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "basis must be between 0 and 4".to_string(),
            );
        }
        if issue_serial >= settlement_serial {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "issue must be before settlement".to_string(),
            );
        }

        let system = self.date_system();
        let days = (|| -> Result<(Day, Day, Day), String> {
            Ok((
                day_of(issue_serial, system)?,
                day_of(first_interest_serial, system)?,
                day_of(settlement_serial, system)?,
            ))
        })();
        let (issue, first_interest, settlement) = match days {
            Ok(d) => d,
            Err(e) => return CalcResult::new_error(Error::NUM, cell, e),
        };

        match compute_accrint(
            system,
            issue,
            first_interest,
            settlement,
            rate,
            par,
            frequency,
            basis,
            calc_method,
        ) {
            Ok(value) => CalcResult::Number(value),
            Err(e) => CalcResult::new_error(Error::NUM, cell, e),
        }
    }
}
