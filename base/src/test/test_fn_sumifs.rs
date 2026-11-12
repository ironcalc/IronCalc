#![allow(clippy::unwrap_used)]

use crate::model::Model;
use crate::test::util::new_empty_model;

#[test]
fn test_fn_sumifs_arguments() {
    let mut model = new_empty_model();

    // Incorrect number of arguments
    model._set("A1", "=SUMIFS()");
    model._set("A2", "=SUMIFS(B2:B9)");
    model._set("A3", "=SUMIFS(B2:B9,C2:C9)");
    model._set("A4", "=SUMIFS(B2:B9,C2:C9,\"=A*\",D2:D9)");

    // Correct (Sum everything in column 'B' if column 'C' starts with "A")
    model._set("A5", "=SUMIFS(B2:B9,C2:C9,\"=A*\")");

    // Data
    model._set("B2", "5");
    model._set("B3", "4");
    model._set("B4", "15");
    model._set("B5", "22");
    model._set("B6", "=NA()");
    model._set("C2", "Apples");
    model._set("C3", "Bananas");
    model._set("C4", "Almonds");
    model._set("C5", "Yoni");
    model._set("C6", "Mandarin");

    model.evaluate();

    // Error (Incorrect number of arguments)
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    // Correct
    assert_eq!(model._get_text("A5"), *"20");
}

// Date-string criteria reach build_criteria via two distinct paths:
//   * fn_countifs has its own criteria loop (covered by test_fn_count.rs)
//   * apply_ifs is the shared path behind SUMIF, SUMIFS, AVERAGEIF,
//     AVERAGEIFS, MAXIFS, MINIFS — exercised here.
//
// Layout:
//   B2..B8  = 45131..45137 (2023-07-24 .. 2023-07-30, all "<7/31/2023")
//   B9      = 45138        (2023-07-31, the criterion target)
//   B10..B11= 45139, 45200 (after 7/31/2023)
//   C2..C8  = 1..7         (matches "<7/31/2023")
//   C9      = 100          (matches "7/31/2023")
//   C10..C11= 99, 50       (matches ">7/31/2023")
fn populate_date_apply_ifs_data(model: &mut Model) {
    let date_serials = [45131, 45132, 45133, 45134, 45135, 45136, 45137];
    let values = [1, 2, 3, 4, 5, 6, 7];
    for (idx, (serial, value)) in date_serials.iter().zip(values.iter()).enumerate() {
        let row = idx + 2;
        model._set(&format!("B{row}"), &serial.to_string());
        model._set(&format!("C{row}"), &value.to_string());
    }
    model._set("B9", "45138");
    model._set("C9", "100");
    model._set("B10", "45139");
    model._set("C10", "99");
    model._set("B11", "45200");
    model._set("C11", "50");
}

#[test]
fn test_apply_ifs_date_criterion_covers_all_consumers() {
    // One worksheet, one criterion shape, every apply_ifs consumer probed.
    // If a future change drops the locale at the apply_ifs call site, every
    // assertion below collapses to 0 / DIV-by-0 and the test fails loudly.
    let mut model = new_empty_model();
    populate_date_apply_ifs_data(&mut model);

    // SUMIF / SUMIFS — sum of C where B "<7/31/2023" is 1+2+3+4+5+6+7 = 28
    model._set("A1", "=SUMIF(B2:B11, \"<7/31/2023\", C2:C11)");
    model._set("A2", "=SUMIFS(C2:C11, B2:B11, \"<7/31/2023\")");
    // AVERAGEIF / AVERAGEIFS — 28 / 7 = 4
    model._set("A3", "=AVERAGEIF(B2:B11, \"<7/31/2023\", C2:C11)");
    model._set("A4", "=AVERAGEIFS(C2:C11, B2:B11, \"<7/31/2023\")");
    // MAXIFS / MINIFS over the same range
    model._set("A5", "=MAXIFS(C2:C11, B2:B11, \"<7/31/2023\")");
    model._set("A6", "=MINIFS(C2:C11, B2:B11, \"<7/31/2023\")");

    // Use a different operator (>=) on SUMIFS / MAXIFS / MINIFS to exercise
    // a second branch of build_criteria through the apply_ifs path.
    // C cells where B >= 7/31/2023 are C9=100, C10=99, C11=50 -> sum 249
    model._set("A7", "=SUMIFS(C2:C11, B2:B11, \">=7/31/2023\")");
    model._set("A8", "=MAXIFS(C2:C11, B2:B11, \">=7/31/2023\")");
    model._set("A9", "=MINIFS(C2:C11, B2:B11, \">=7/31/2023\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"28");
    assert_eq!(model._get_text("A2"), *"28");
    assert_eq!(model._get_text("A3"), *"4");
    assert_eq!(model._get_text("A4"), *"4");
    assert_eq!(model._get_text("A5"), *"7");
    assert_eq!(model._get_text("A6"), *"1");
    assert_eq!(model._get_text("A7"), *"249");
    assert_eq!(model._get_text("A8"), *"100");
    assert_eq!(model._get_text("A9"), *"50");
}

#[test]
fn test_apply_ifs_date_criterion_respects_locale() {
    // Build the same workbook under en-GB (D/M/Y). "31/7/2023" is only a
    // valid date under a D/M/Y locale; if apply_ifs ever stops forwarding
    // self.locale to build_criteria, en's M/D/Y rules would reject this
    // string and SUMIFS would collapse to 0.
    let mut model = Model::new_empty("model", "en-GB", "UTC", "en").unwrap();
    populate_date_apply_ifs_data(&mut model);

    model._set("A1", "=SUMIFS(C2:C11, B2:B11, \"<31/7/2023\")");
    model._set("A2", "=AVERAGEIFS(C2:C11, B2:B11, \"<31/7/2023\")");
    model._set("A3", "=MAXIFS(C2:C11, B2:B11, \"<31/7/2023\")");
    model._set("A4", "=MINIFS(C2:C11, B2:B11, \"<31/7/2023\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"28");
    assert_eq!(model._get_text("A2"), *"4");
    assert_eq!(model._get_text("A3"), *"7");
    assert_eq!(model._get_text("A4"), *"1");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn test_fn_sumifs_whole_column_sees_a_spill_beyond_the_used_area() {
    let cases = [
        ("SUMIF(D:D,\">2\")", "12"),
        ("SUMIFS(D:D,D:D,\">2\")", "12"),
        // The sum range takes the size of C:C: D:D, and D2:D1048576
        ("SUMIF(C:C,\">0\",D1)", "6"),
        ("SUMIF(C:C,\">0\",D2)", "9"),
    ];
    for (formula, expected) in cases {
        let mut model = new_empty_model();
        model._set("A1", &format!("=SEQUENCE(1,1,{formula})"));
        model._set("C1", "=SEQUENCE(3,1,10)");
        model._set("D1", "=SEQUENCE(5)");
        model.evaluate();
        assert_eq!(model._get_text("A1"), expected, "{formula}");
        assert_eq!(model.evaluation.restarts_in_last_evaluation, 1, "{formula}");
    }
}

// ── ranges of different sizes ────────────────────────────────────────────────

// COUNTIFS, SUMIFS, AVERAGEIFS, MAXIFS and MINIFS need ranges of the same
// size, as in Excel. SUMIF and AVERAGEIF do not.
#[test]
fn test_fn_ifs_ranges_of_different_sizes() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "x");
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");

    model._set("D1", "=SUMIFS(B1:B3,A1:A2,\"x\")");
    // The second criteria range
    model._set("D2", "=SUMIFS(B1:B3,A1:A3,\"x\",A1,\"x\")");
    model._set("D3", "=AVERAGEIFS(B1:B3,A1,\"x\")");
    model._set("D4", "=MAXIFS(B1,A1:A3,\"x\")");
    // Three rows each, but one column and two
    model._set("D5", "=MINIFS(B1:B3,A1:B3,\"x\")");
    model._set("D6", "=COUNTIFS(A1:A3,\"x\",B1,\">5\")");
    model._set("D7", "=COUNTIFS(A1:A3,\"x\",B1:C3,\">5\")");

    // Ranges of the same size
    model._set("E1", "=SUMIFS(B1:B3,A1:A3,\"x\")");
    model._set("E2", "=COUNTIFS(A1:A3,\"x\",B1:B3,\">5\")");
    // SUMIF and AVERAGEIF with a longer sum range
    model._set("E3", "=SUMIF(A1:A3,\"x\",B1:B5)");
    model._set("E4", "=AVERAGEIF(A1:A3,\"x\",B1:B5)");

    model.evaluate();

    for cell in ["D1", "D2", "D3", "D4", "D5", "D6", "D7"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
    assert_eq!(model._get_text("E1"), *"40");
    assert_eq!(model._get_text("E2"), *"2");
    assert_eq!(model._get_text("E3"), *"40");
    assert_eq!(model._get_text("E4"), *"20");
}

// ── SUMIF and AVERAGEIF: the sum range takes the size of the criteria range ──

// It starts at the top left cell of the sum range given, as in Excel.
#[test]
fn test_fn_sumif_sum_range_takes_the_size_of_the_criteria_range() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "x");
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");
    model._set("C1", "1");
    model._set("D1", "2");
    model._set("E1", "3");

    // The sum range is B1:B3
    model._set("G1", "=SUMIF(A1:A3,\"x\",B1)");
    model._set("G2", "=AVERAGEIF(A1:A3,\"x\",B1)");
    // The sum range is B1
    model._set("G3", "=SUMIF(A1,\"x\",B1:B3)");
    model._set("G4", "=AVERAGEIF(A1,\"x\",B1:B3)");
    // One column wide, as B1:B3: C1:C3, so D1 and E1 are not added
    model._set("G5", "=SUMIF(B1:B3,\">0\",C1:E1)");
    // Two columns wide, as A1:B3: C1:D3, where B1 goes with D1
    model._set("G6", "=SUMIF(A1:B3,\">5\",C1)");
    // An array criterion spills one sum per criterion
    model._set("G7", "=SUMIF(A1:A3,{\"x\",\"y\"},B1)");

    model.evaluate();

    assert_eq!(model._get_text("G1"), *"40");
    assert_eq!(model._get_text("G2"), *"20");
    assert_eq!(model._get_text("G3"), *"10");
    assert_eq!(model._get_text("G4"), *"10");
    assert_eq!(model._get_text("G5"), *"1");
    assert_eq!(model._get_text("G6"), *"2");
    assert_eq!(model._get_text("G7"), *"40");
    assert_eq!(model._get_text("H7"), *"20");
}

// With the criteria range A:A the sum range is B5:B1048576, and with 1:1 it is
// B5:XFD5. It is walked only as far as the used area goes, and the rest is
// one rectangle on record, not a record per cell.
#[test]
fn test_fn_sumif_resized_sum_range_is_clipped_to_the_used_area() {
    let cases = [
        ("SUMIF(A:A,\"x\",B5)", ["A1", "A2", "B5", "B6"]),
        ("SUMIF(1:1,\"x\",B5)", ["A1", "B1", "B5", "C5"]),
    ];
    for (formula, cells) in cases {
        let mut model = new_empty_model();
        model._set(cells[0], "x");
        model._set(cells[1], "x");
        model._set(cells[2], "5");
        model._set(cells[3], "6");
        // Reads are recorded for an anchor's spill to be checked against: with
        // no dynamic anchor in the workbook none is kept. This one reads nothing.
        model._set("F3", "=SEQUENCE(1)");
        model._set("H3", &format!("={formula}"));

        model.evaluate();

        assert_eq!(model._get_text("H3"), *"11", "{formula}");
        assert!(
            model.evaluation.seen.len() <= 12,
            "{formula}: {}",
            model.evaluation.seen.len()
        );
        assert_eq!(model.evaluation.seen_ranges.len(), 1, "{formula}");
    }
}

// Only the sum range itself is on record: here it starts beyond the used area,
// at B500 and at Z2. The formula spills short of it, and is not circular.
#[test]
fn test_fn_sumif_resized_sum_range_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("B10", "=SEQUENCE(3,1,SUMIF(A:A,\"x\",B500))");
    model.evaluate();
    assert_eq!(model._get_text("B10"), *"0");
    assert_eq!(model._get_text("B12"), *"2");

    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("J2", "=SEQUENCE(1,3,SUMIF(1:1,\"x\",Z2))");
    model.evaluate();
    assert_eq!(model._get_text("J2"), *"0");
    assert_eq!(model._get_text("L2"), *"2");
}
