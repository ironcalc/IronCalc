#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::compare::compare;
use ironcalc::export::save_to_xlsx;
use ironcalc::import::load_from_xlsx;
use ironcalc_base::expressions::utils::number_to_column;
use ironcalc_base::Model;

fn save_and_reload(model: &Model) -> Model<'static> {
    let dir = std::env::temp_dir().join(format!("{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("tables.xlsx");
    let file = file.to_str().unwrap();
    save_to_xlsx(model, file).unwrap();
    let mut reloaded = load_from_xlsx(file, "en", "UTC", "en").unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    reloaded.evaluate();
    reloaded
}

fn sheet_index(model: &Model, name: &str) -> u32 {
    let names = model.workbook.get_worksheet_names();
    names.iter().position(|n| n == name).unwrap() as u32
}

fn formula(model: &Model, sheet: &str, row: i32, column: i32) -> String {
    let sheet = sheet_index(model, sheet);
    model.get_cell_formula(sheet, row, column).unwrap().unwrap()
}

// The export writes no table definitions yet, so it writes each structured
// reference as the cells it points to, and the saved workbook calculates the
// same when it is read again.
#[test]
fn tables_calculate_the_same_after_saving() {
    let model = load_from_xlsx("tests/calc_test_no_export/tables.xlsx", "en", "UTC", "en").unwrap();
    let reloaded = save_and_reload(&model);

    // SUBTOTAL(1-11, ...) skips the rows a table's filter hides, and counts
    // rows hidden by hand. Without the table in the file, the filtered rows of
    // these tables read back as rows hidden by hand.
    let filtered_rows_read_back_as_hidden = [
        "filters!B8",
        "filters!B10",
        "Functions Filters!D46",
        "Functions Filters!D47",
        "Functions Filters!E48",
        "Functions Filters!D50",
        "Functions Filters!D54",
    ];
    let differences: Vec<String> = match compare(&model, &reloaded) {
        Ok(diffs) => diffs
            .iter()
            .map(|d| {
                let column = number_to_column(d.column).unwrap();
                format!("{}!{column}{}", d.sheet_name, d.row)
            })
            .collect(),
        Err(_) => panic!("The saved workbook has other sheets"),
    };
    assert_eq!(differences, filtered_rows_read_back_as_hidden);

    // The formula bar still shows the structured references
    assert_eq!(formula(&model, "Sheet1", 5, 1), "=SHEET(Table1)");
    assert_eq!(
        formula(&model, "Escape", 2, 5),
        "=SUBTOTAL(109,Table9[@[Hi, ''This]:['[Not me']]])"
    );
    // and the saved workbook has the cells they point to: absolute, but for
    // the row of `[@Column]`, which follows the cell
    assert_eq!(
        formula(&reloaded, "Sheet1", 5, 1),
        "=SHEET(WithHeaders!$A$2:$F$4)"
    );
    assert_eq!(formula(&reloaded, "Escape", 2, 5), "=SUBTOTAL(109,$B2:$D2)");
}
