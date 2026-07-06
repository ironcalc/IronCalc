#![allow(clippy::unwrap_used)]

use crate::{
    cell::CellValue,
    task::{FinanceFetchTask, Task},
    test::util::new_empty_model,
};

#[test]
fn test_currencies() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "$100.348".to_string())
        .unwrap();
    model
        .set_user_input(0, 1, 2, "=ISNUMBER(A1)".to_string())
        .unwrap();

    model
        .set_user_input(0, 2, 1, "$ 100.348".to_string())
        .unwrap();
    model
        .set_user_input(0, 2, 2, "=ISNUMBER(A2)".to_string())
        .unwrap();

    model.set_user_input(0, 3, 1, "100$".to_string()).unwrap();
    model
        .set_user_input(0, 3, 2, "=ISNUMBER(A3)".to_string())
        .unwrap();

    model
        .set_user_input(0, 4, 1, "3.1415926$".to_string())
        .unwrap();

    model.evaluate();

    // two decimal rounded up
    assert_eq!(model._get_text("A1"), "$100.35");
    assert_eq!(model._get_text("B1"), *"TRUE");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(100.348))
    );
    // No space
    assert_eq!(model._get_text("A2"), "$100.35");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A2"),
        Ok(CellValue::Number(100.348))
    );
    assert_eq!(model._get_text("B2"), *"TRUE");

    // Dollar is on the right
    assert_eq!(model._get_text("A3"), "100$");
    assert_eq!(model._get_text("B3"), *"TRUE");

    assert_eq!(model._get_text("A4"), "3.14$");
}

#[test]
fn scientific() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "3e-4".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "5e-4$".to_string()).unwrap();
    model.set_user_input(0, 3, 1, "6e-4%".to_string()).unwrap();

    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(0.0003))
    );
    assert_eq!(model._get_text("Sheet1!A1"), "3.00E-04");
    assert_eq!(model._get_text("Sheet1!A2"), "5.00E-04");
    assert_eq!(model._get_text("Sheet1!A3"), "6.00E-06");
}

#[test]
fn test_percentage() {
    let mut model = new_empty_model();
    model.set_user_input(0, 10, 1, "50%".to_string()).unwrap();
    model
        .set_user_input(0, 10, 2, "=ISNUMBER(A10)".to_string())
        .unwrap();
    model
        .set_user_input(0, 11, 1, "55.759%".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(model._get_text("B10"), *"TRUE");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A10"),
        Ok(CellValue::Number(0.5))
    );
    // Two decimal places
    assert_eq!(model._get_text("A11"), "55.76%");
}

#[test]
fn test_percentage_ops() {
    let mut model = new_empty_model();
    model._set("A1", "5%");
    model._set("A2", "20%");
    model.set_user_input(0, 3, 1, "=A1+A2".to_string()).unwrap();
    model.set_user_input(0, 4, 1, "=A1*A2".to_string()).unwrap();

    model.evaluate();

    assert_eq!(model._get_text("A3"), *"25%");
    assert_eq!(model._get_text("A4"), *"1.00%");
}

#[test]
fn test_numbers() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "1,000,000".to_string())
        .unwrap();

    model
        .set_user_input(0, 20, 1, "50,123.549".to_string())
        .unwrap();
    model
        .set_user_input(0, 21, 1, "50,12.549".to_string())
        .unwrap();
    model
        .set_user_input(0, 22, 1, "1,234567".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(1000000.0))
    );

    // Two decimal places
    assert_eq!(model._get_text("A20"), "50,123.55");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A20"),
        Ok(CellValue::Number(50123.549))
    );

    // This is a string
    assert_eq!(model._get_text("A21"), "50,12.549");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A21"),
        Ok(CellValue::String("50,12.549".to_string()))
    );

    // Commas in all places
    assert_eq!(model._get_text("A22"), "1,234,567");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A22"),
        Ok(CellValue::Number(1234567.0))
    );
}

#[test]
fn test_negative_numbers() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "-100".to_string()).unwrap();

    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(-100.0))
    );
}

#[test]
fn test_negative_currencies() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "-$100".to_string()).unwrap();
    model
        .set_user_input(0, 2, 1, "-$99.123".to_string())
        .unwrap();
    // This is valid!
    model.set_user_input(0, 3, 1, "$-345".to_string()).unwrap();

    model.set_user_input(0, 1, 2, "-200$".to_string()).unwrap();
    model
        .set_user_input(0, 2, 2, "-92.689$".to_string())
        .unwrap();
    // This is valid!
    model.set_user_input(0, 3, 2, "-22$".to_string()).unwrap();

    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(-100.0))
    );
    assert_eq!(model._get_text("A1"), *"-$100");
    assert_eq!(model._get_text("A2"), *"-$99.12");
    assert_eq!(model._get_text("A3"), *"-$345");

    assert_eq!(model._get_text("B1"), *"-200$");
    assert_eq!(model._get_text("B2"), *"-92.69$");
    assert_eq!(model._get_text("B3"), *"-22$");
}

#[test]
fn test_formulas() {
    let mut model = new_empty_model();
    model._set("A1", "$100");
    model._set("A2", "$200");
    model.set_user_input(0, 3, 1, "=A1+A2".to_string()).unwrap();
    model
        .set_user_input(0, 4, 1, "=SUM(A1:A3)".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(model._get_text("A3"), *"$300");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A3"),
        Ok(CellValue::Number(300.0))
    );
    assert_eq!(model._get_text("A4"), *"$600");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A4"),
        Ok(CellValue::Number(600.0))
    );
}

#[test]
fn test_product() {
    let mut model = new_empty_model();
    model._set("A1", "$100");
    model._set("A2", "$5");
    model._set("A3", "4");

    model.set_user_input(0, 1, 2, "=A1*A2".to_string()).unwrap();
    model.set_user_input(0, 2, 2, "=A1*A3".to_string()).unwrap();
    model.set_user_input(0, 3, 2, "=A1*3".to_string()).unwrap();

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"500");
    assert_eq!(model._get_text("B2"), *"$400");
    assert_eq!(model._get_text("B3"), *"$300");
}

#[test]
fn test_division() {
    let mut model = new_empty_model();
    model._set("A1", "$100");
    model._set("A2", "$5");
    model._set("A3", "4");

    model.set_user_input(0, 1, 2, "=A1/A2".to_string()).unwrap();
    model.set_user_input(0, 2, 2, "=A1/A3".to_string()).unwrap();
    model.set_user_input(0, 3, 2, "=A1/2".to_string()).unwrap();
    model
        .set_user_input(0, 4, 2, "=100/A2".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"20");
    assert_eq!(model._get_text("B2"), *"$25");
    assert_eq!(model._get_text("B3"), *"$50");
    assert_eq!(model._get_text("B4"), *"20");
}

#[test]
fn test_some_complex_examples() {
    let mut model = new_empty_model();
    // $3.00 / 2 = $1.50
    model._set("A1", "$3.00");
    model._set("A2", "2");
    model.set_user_input(0, 3, 1, "=A1/A2".to_string()).unwrap();

    // $3 / 2 = $1
    model._set("B1", "$3");
    model._set("B2", "2");
    model.set_user_input(0, 3, 2, "=B1/B2".to_string()).unwrap();

    // $5.00 * 25% = 25% * $5.00 = $1.25
    model._set("C1", "$5.00");
    model._set("C2", "25%");
    model.set_user_input(0, 3, 3, "=C1*C2".to_string()).unwrap();
    model.set_user_input(0, 4, 3, "=C2*C1".to_string()).unwrap();

    // $5 * 75% = 75% * $5 = $1
    model._set("D1", "$5");
    model._set("D2", "75%");
    model.set_user_input(0, 3, 4, "=D1*D2".to_string()).unwrap();
    model.set_user_input(0, 4, 4, "=D2*D1".to_string()).unwrap();

    // $10 + $9.99 = $9.99 + $10 = $19.99
    model._set("E1", "$10");
    model._set("E2", "$9.99");
    model.set_user_input(0, 3, 5, "=E1+E2".to_string()).unwrap();
    model.set_user_input(0, 4, 5, "=E2+E1".to_string()).unwrap();

    // $2 * 2 = 2 * $2 = $4
    model._set("F1", "$2");
    model._set("F2", "2");
    model.set_user_input(0, 3, 6, "=F1*F2".to_string()).unwrap();
    model.set_user_input(0, 4, 6, "=F2*F1".to_string()).unwrap();

    // $2.50 * 2 = 2 * $2.50 = $5.00
    model._set("G1", "$2.50");
    model._set("G2", "2");
    model.set_user_input(0, 3, 7, "=G1*G2".to_string()).unwrap();
    model.set_user_input(0, 4, 7, "=G2*G1".to_string()).unwrap();

    // $2 * 2.5 = 2.5 * $2 = $5
    model._set("H1", "$2");
    model._set("H2", "2.5");
    model.set_user_input(0, 3, 8, "=H1*H2".to_string()).unwrap();
    model.set_user_input(0, 4, 8, "=H2*H1".to_string()).unwrap();

    // 10% * 1,000 = 1,000 * 10% = 100
    model._set("I1", "10%");
    model._set("I2", "1,000");
    model.set_user_input(0, 3, 9, "=I1*I2".to_string()).unwrap();
    model.set_user_input(0, 4, 9, "=I2*I1".to_string()).unwrap();

    model.evaluate();

    assert_eq!(model._get_text("A3"), *"$1.50");

    assert_eq!(model._get_text("B3"), *"$2");

    assert_eq!(model._get_text("C3"), *"$1.25");
    assert_eq!(model._get_text("C4"), *"$1.25");

    assert_eq!(model._get_text("D3"), *"$3.75");
    assert_eq!(model._get_text("D4"), *"$3.75");

    assert_eq!(model._get_text("E3"), *"$19.99");
    assert_eq!(model._get_text("E4"), *"$19.99");

    assert_eq!(model._get_text("F3"), *"$4");
    assert_eq!(model._get_text("F4"), *"$4");

    assert_eq!(model._get_text("G3"), *"$5.00");
    assert_eq!(model._get_text("G4"), *"$5.00");

    assert_eq!(model._get_text("H3"), *"$5");
    assert_eq!(model._get_text("H4"), *"$5");

    assert_eq!(model._get_text("I3"), *"100");
    assert_eq!(model._get_text("I4"), *"100");
}

#[test]
fn test_financial_functions() {
    // Some functions imply a currency formatting even on error
    let mut model = new_empty_model();
    model._set("A2", "8%");
    model._set("A3", "10");
    model._set("A4", "$10,000");

    model
        .set_user_input(0, 5, 1, "=PMT(A2/12,A3,A4)".to_string())
        .unwrap();
    model
        .set_user_input(0, 6, 1, "=PMT(A2/12,A3,A4,,1)".to_string())
        .unwrap();
    model
        .set_user_input(0, 7, 1, "=PMT(0.2, 3, -200)".to_string())
        .unwrap();

    model.evaluate();

    // This two are negative numbers
    assert_eq!(model._get_text("A5"), *"-$1,037.03");
    assert_eq!(model._get_text("A6"), *"-$1,030.16");
    // This is a positive number
    assert_eq!(model._get_text("A7"), *"$94.95");
}

#[test]
fn test_finance_function() {
    // FINANCE(ticker, attribute, asset_type)
    //
    // Uses the Elm-like side-effect architecture:
    //   1. On cache miss → take_tasks() returns a Task, cell shows #N/A.
    //   2. Caller completes the task → populates the cache.
    //   3. Re-evaluate → cache hits → cell shows the actual value.
    let mut model = new_empty_model();

    // Fewer than 3 args → argument count error
    model._set("A1", "=FINANCE()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");

    // Still fewer than 3
    model._set("A2", "=FINANCE(\"AAPL\", \"price\")");
    model.evaluate();
    assert_eq!(model._get_text("A2"), *"#ERROR!");

    // 3 args but no cache → returns a Task, cell shows loading
    model._set("A3", "=FINANCE(\"AAPL\", \"price\", \"stock\")");
    model.evaluate();
    let tasks = model.take_tasks();
    assert_eq!(model._get_text("A3"), *"#N/A");
    assert_eq!(tasks.len(), 1);

    let finance_task = match &tasks[0] {
        Task::FinanceFetch(task) => task,
    };
    assert_eq!(finance_task.ticker.symbol(), "AAPL");
    assert_eq!(finance_task.attribute, "price");

    // Complete the task → cache gets populated
    model.complete_financial_task(finance_task.clone(), Ok(195.89));

    // Re-evaluate → cache hit → cell shows the number
    model.evaluate();
    let tasks = model.take_tasks();
    assert!(tasks.is_empty());
    assert_eq!(model._get_text("A3"), "195.89");

    // Second ticker — cache miss
    model._set("A4", "=FINANCE(\"MSFT\", \"open\", \"stock\")");
    model.evaluate();
    let tasks = model.take_tasks();
    assert_eq!(tasks.len(), 1);
    assert_eq!(model._get_text("A4"), *"#N/A");
    let finance_task = match &tasks[0] {
        Task::FinanceFetch(task) => task,
    };

    // Complete with an error result
    use crate::finance::provider::FinanceError;
    model.complete_financial_task(
        finance_task.clone(),
        Err(FinanceError::TickerNotFound("MSFT".into())),
    );

    // Re-evaluate → cache hit with error → cell shows #N/A with error message
    model.evaluate();
    let tasks = model.take_tasks();
    assert!(tasks.is_empty());
    assert_eq!(model._get_text("A4"), *"#N/A");

    // Unknown asset type → #VALUE!
    model._set("A5", "=FINANCE(\"AAPL\", \"price\", \"bonds\")");
    model.evaluate();
    let tasks = model.take_tasks();
    assert!(tasks.is_empty());
    assert_eq!(model._get_text("A5"), *"#VALUE!");

    // Verify that completing the same task again is idempotent
    // (just overwrites the cache entry)
    model._set("A6", "=FINANCE(\"AAPL\", \"price\", \"stock\")");
    model.evaluate();
    let tasks = model.take_tasks();
    assert!(tasks.is_empty()); // already cached
    assert_eq!(model._get_text("A6"), "195.89");
}

#[test]
fn test_sum_function() {
    let mut model = new_empty_model();
    model._set("A1", "$100");
    model._set("A2", "$300");

    model
        .set_user_input(0, 1, 2, "=SUM(A:A)".to_string())
        .unwrap();
    model
        .set_user_input(0, 2, 2, "=SUM(A1:A2)".to_string())
        .unwrap();
    model
        .set_user_input(0, 3, 2, "=SUM(A1, A2, A3)".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"$400");
    assert_eq!(model._get_text("B2"), *"$400");
    assert_eq!(model._get_text("B3"), *"$400");
}

#[test]
fn test_number() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "3".to_string()).unwrap();

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(3.0))
    );
}

#[test]
fn test_currencies_eur_prefix() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "€100.348".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(model._get_text("A1"), "€100.35");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(100.348))
    );
}

#[test]
fn test_currencies_eur_suffix() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "100.348€".to_string())
        .unwrap();
    model.set_user_input(0, 2, 1, "25€".to_string()).unwrap();

    // negatives
    model
        .set_user_input(0, 1, 2, "-123.348€".to_string())
        .unwrap();
    model.set_user_input(0, 2, 2, "-42€".to_string()).unwrap();

    // with a space
    model
        .set_user_input(0, 1, 3, "101.348 €".to_string())
        .unwrap();
    model.set_user_input(0, 2, 3, "26 €".to_string()).unwrap();

    model
        .set_user_input(0, 1, 4, "-12.348 €".to_string())
        .unwrap();
    model.set_user_input(0, 2, 4, "-45 €".to_string()).unwrap();

    model.evaluate();

    assert_eq!(model._get_text("A1"), "100.35€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(100.348))
    );
    assert_eq!(model._get_text("A2"), "25€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A2"),
        Ok(CellValue::Number(25.0))
    );

    assert_eq!(model._get_text("B1"), "-123.35€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(-123.348))
    );
    assert_eq!(model._get_text("B2"), "-42€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B2"),
        Ok(CellValue::Number(-42.0))
    );

    // with a space
    assert_eq!(model._get_text("C1"), "101.35€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!C1"),
        Ok(CellValue::Number(101.348))
    );
    assert_eq!(model._get_text("C2"), "26€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!C2"),
        Ok(CellValue::Number(26.0))
    );

    assert_eq!(model._get_text("D1"), "-12.35€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!D1"),
        Ok(CellValue::Number(-12.348))
    );
    assert_eq!(model._get_text("D2"), "-45€");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!D2"),
        Ok(CellValue::Number(-45.0))
    );
}

#[test]
fn test_sum_function_eur() {
    let mut model = new_empty_model();
    model._set("A1", "€100");
    model._set("A2", "€300");

    model
        .set_user_input(0, 1, 2, "=SUM(A:A)".to_string())
        .unwrap();
    model
        .set_user_input(0, 2, 2, "=SUM(A1:A2)".to_string())
        .unwrap();
    model
        .set_user_input(0, 3, 2, "=SUM(A1, A2, A3)".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"€400");
    assert_eq!(model._get_text("B2"), *"€400");
    assert_eq!(model._get_text("B3"), *"€400");
}

#[test]
fn input_dates() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "4/3/2025".to_string())
        .unwrap();

    model.evaluate();

    assert_eq!(model._get_text("A1"), "4/3/2025");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );

    // further date assignments do not change the format
    model
        .set_user_input(0, 1, 1, "08-08-2028".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "8/8/2028");
}
