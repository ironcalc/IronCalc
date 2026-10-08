#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn mmult_2x2() {
    let mut model = new_empty_model();
    // A = [[1, 2],
    //      [3, 4]]
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    // B = [[5, 6],
    //      [7, 8]]
    model._set("D1", "5");
    model._set("E1", "6");
    model._set("D2", "7");
    model._set("E2", "8");

    model._set("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    // A*B = [[19, 22],
    //        [43, 50]]
    assert_eq!(model._get_text("G1"), "19");
    assert_eq!(model._get_text("H1"), "22");
    assert_eq!(model._get_text("G2"), "43");
    assert_eq!(model._get_text("H2"), "50");
}

#[test]
fn mmult_2x3_times_3x2() {
    let mut model = new_empty_model();
    // A = [[1, 2, 3],
    //      [4, 5, 6]]
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");
    // B = [[7,  8],
    //      [9,  10],
    //      [11, 12]]
    model._set("E1", "7");
    model._set("F1", "8");
    model._set("E2", "9");
    model._set("F2", "10");
    model._set("E3", "11");
    model._set("F3", "12");

    model._set("H1", "=MMULT(A1:C2, E1:F3)");
    model.evaluate();
    // A*B = [[58,  64],
    //        [139, 154]]
    assert_eq!(model._get_text("H1"), "58");
    assert_eq!(model._get_text("I1"), "64");
    assert_eq!(model._get_text("H2"), "139");
    assert_eq!(model._get_text("I2"), "154");
}

#[test]
fn mmult_dimension_mismatch() {
    let mut model = new_empty_model();
    // 1x2 times 3x1 -> mismatch
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("D1", "3");
    model._set("D2", "4");
    model._set("D3", "5");
    model._set("F1", "=MMULT(A1:B1, D1:D3)");
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#VALUE!");
}

#[test]
fn mmult_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=MMULT(1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn mmult_non_numeric() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "\"x\"");
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("F1", "=MMULT(A1:B1, D1:D2)");
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#VALUE!");
}

#[test]
fn mmult_empty_cell_in_input() {
    // An empty cell inside either matrix must yield #VALUE!
    let mut model = new_empty_model();
    model._set("A1", "1");
    // B1 left empty
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "5");
    model._set("E1", "6");
    model._set("D2", "7");
    model._set("E2", "8");
    model._set("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    assert_eq!(model._get_text("G1"), "#VALUE!");
}

#[test]
fn mmult_boolean_in_input() {
    // Booleans are not coerced for MMULT — they yield #VALUE!.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "TRUE");
    model._set("A2", "3");
    model._set("B2", "FALSE");
    model._set("D1", "5");
    model._set("E1", "6");
    model._set("D2", "7");
    model._set("E2", "8");
    model._set("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    assert_eq!(model._get_text("G1"), "#VALUE!");
}

#[test]
fn mmult_omitted_arguments() {
    let mut model = new_empty_model();
    model._set("C1", "1");
    model._set("D1", "2");
    model._set("C2", "3");
    model._set("D2", "4");
    model._set("A1", "=MMULT(,C1:D2)");
    model._set("A2", "=MMULT(,)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#VALUE!");
}

#[test]
fn mmult_overflow_returns_num() {
    // Two 1x1 matrices whose product overflows f64.
    let mut model = new_empty_model();
    model._set("A1", "1E300");
    model._set("C1", "1E300");
    model._set("E1", "=MMULT(A1, C1)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "#NUM!");
}

#[test]
fn mmult_propagates_input_error() {
    // An error in either argument must be propagated (not turned into #VALUE!).
    let mut model = new_empty_model();
    model._set("A1", "=NA()");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "5");
    model._set("E1", "6");
    model._set("D2", "7");
    model._set("E2", "8");
    model._set("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    assert_eq!(model._get_text("G1"), "#N/A");
}

// A is 2x3 and B is 3x2, each also written as its transpose:
//      A: A1:C2    its transpose: A4:B6
//      B: E1:F3    its transpose: E5:G6
fn model_with_two_matrices() -> crate::Model<'static> {
    let mut model = new_empty_model();
    let a = [[1.5, 2.0, -3.0], [4.0, 0.25, 6.0]];
    let b = [[7.0, 8.0], [-9.0, 10.5], [11.0, 12.0]];
    for (i, row) in a.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            let column = (b'A' + j as u8) as char;
            model._set(&format!("{column}{}", i + 1), &value.to_string());
            // the transpose, from A4
            let column = (b'A' + i as u8) as char;
            model._set(&format!("{column}{}", j + 4), &value.to_string());
        }
    }
    for (i, row) in b.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            let column = (b'E' + j as u8) as char;
            model._set(&format!("{column}{}", i + 1), &value.to_string());
            // the transpose, from E5
            let column = (b'E' + i as u8) as char;
            model._set(&format!("{column}{}", j + 5), &value.to_string());
        }
    }
    model
}

#[test]
fn mmult_of_transposed_ranges() {
    let mut model = model_with_two_matrices();
    // the 2x2 product, four ways
    model._set("I1", "=MMULT(A1:C2, E1:F3)");
    model._set("L1", "=MMULT(A1:C2, TRANSPOSE(E5:G6))");
    model._set("O1", "=MMULT(TRANSPOSE(A4:B6), E1:F3)");
    model._set(
        "R1",
        "=MMULT(TRANSPOSE(A4:B6), TRANSPOSE(TRANSPOSE(TRANSPOSE(E5:G6))))",
    );
    // the 3x3 product the other way around, and its transpose
    model._set("I5", "=MMULT(E1:F3, A1:C2)");
    model._set("M5", "=MMULT(TRANSPOSE(E5:G6), TRANSPOSE(A4:B6))");
    model._set(
        "Q5",
        "=TRANSPOSE(MMULT(TRANSPOSE(A1:C2), TRANSPOSE(E1:F3)))",
    );
    model.evaluate();
    let expected = [["-40.5", "-3"], ["91.75", "106.625"]];
    for corner in ['I', 'L', 'O', 'R'] {
        for (i, row) in expected.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                let cell = format!("{}{}", (corner as u8 + j as u8) as char, i + 1);
                assert_eq!(model._get_text(&cell), *value, "{cell}");
            }
        }
    }
    let expected = [
        ["42.5", "16", "27"],
        ["28.5", "-15.375", "90"],
        ["64.5", "25", "39"],
    ];
    for corner in ['I', 'M', 'Q'] {
        for (i, row) in expected.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                let cell = format!("{}{}", (corner as u8 + j as u8) as char, i + 5);
                assert_eq!(model._get_text(&cell), *value, "{cell}");
            }
        }
    }
}

#[test]
fn mmult_of_formulas_and_of_arrays() {
    let mut model = model_with_two_matrices();
    // A again, as formulas, and B as the result of a calculation
    for cell in ["A", "B", "C"] {
        model._set(&format!("{cell}10"), &format!("={cell}1*1"));
        model._set(&format!("{cell}11"), &format!("={cell}2+0"));
    }
    model._set("I1", "=MMULT(A10:C11, E1:F3*1)");
    model._set("L1", "=MMULT(A10:C11, TRANSPOSE(E5:G6+0))");
    model.evaluate();
    for cell in ["I1", "L1"] {
        assert_eq!(model._get_text(cell), "-40.5", "{cell}");
    }
    for cell in ["J2", "M2"] {
        assert_eq!(model._get_text(cell), "106.625", "{cell}");
    }
}

#[test]
fn mmult_transposed_dimension_mismatch() {
    let mut model = model_with_two_matrices();
    // 2x3 by 2x3, and 3x2 by 3x2
    model._set("I1", "=MMULT(A1:C2, TRANSPOSE(E1:F3))");
    model._set("I5", "=MMULT(TRANSPOSE(A1:C2), E1:F3)");
    model._set("I9", "=MMULT(TRANSPOSE(A1:C2,1), E1:F3)");
    model.evaluate();
    assert_eq!(model._get_text("I1"), "#VALUE!");
    assert_eq!(model._get_text("I5"), "#VALUE!");
    assert_eq!(model._get_text("I9"), "#ERROR!");
}

// Of the values that are not numbers the first one decides the error, row by
// row: of the matrix as it is multiplied, which is the transposed one.
#[test]
fn mmult_first_value_that_is_not_a_number() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "text");
    model._set("A2", "=1/0");
    model._set("B2", "4");
    model._set("D1", "1");
    model._set("E1", "2");
    model._set("D2", "3");
    model._set("E2", "4");
    model._set("G1", "=MMULT(A1:B2, D1:E2)");
    model._set("G3", "=MMULT(TRANSPOSE(A1:B2), D1:E2)");
    model._set("G5", "=MMULT(D1:E2, A1:B2)");
    model._set("G7", "=MMULT(D1:E2, TRANSPOSE(A1:B2))");
    // nothing in H10
    model._set("G9", "=MMULT(D1:E2, TRANSPOSE(G10:H11))");
    model._set("G10", "1");
    model._set("G11", "3");
    model._set("H11", "4");
    model.evaluate();
    assert_eq!(model._get_text("G1"), "#VALUE!");
    assert_eq!(model._get_text("G3"), "#DIV/0!");
    assert_eq!(model._get_text("G5"), "#VALUE!");
    assert_eq!(model._get_text("G7"), "#DIV/0!");
    assert_eq!(model._get_text("G9"), "#VALUE!");
}
