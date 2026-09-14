use ironcalc_base::collab::model::CollabModel;
use ironcalc_base::Model;

use crate::export::save_xlsx_to_writer;
use crate::import::load_from_xlsx_bytes;

#[test]
fn collab_xlsx_round_trip() {
    let bytes = std::fs::read("tests/example.xlsx").unwrap();
    let load = || load_from_xlsx_bytes(&bytes, "example", "en", "UTC").unwrap();

    let mut ordinal = Model::from_workbook(load(), "en").unwrap();
    ordinal.evaluate();

    let mut collab = CollabModel::from_workbook_with_session(load(), "en", 1).unwrap();
    collab.evaluate();

    // Not a vacuous round trip: the fixture has several sheets and formulas on them.
    assert!(ordinal.workbook.worksheets.len() > 1);
    assert!(collab
        .workbook
        .worksheets
        .iter()
        .any(|ws| !ws.shared_formulas.is_empty()));

    let writer = std::io::Cursor::new(Vec::new());
    let exported = Model::from_workbook(collab.to_ordinal_workbook(), "en").unwrap();
    let out = save_xlsx_to_writer(&exported, writer).unwrap().into_inner();
    let mut reloaded = Model::from_workbook(
        load_from_xlsx_bytes(&out, "example", "en", "UTC").unwrap(),
        "en",
    )
    .unwrap();
    reloaded.evaluate();

    assert_eq!(
        reloaded.workbook.get_worksheet_names(),
        ordinal.workbook.get_worksheet_names()
    );
    for sheet in 0..ordinal.workbook.worksheets.len() as u32 {
        let dimension = ordinal.workbook.worksheet(sheet).unwrap().dimension();
        for row in dimension.min_row..=dimension.max_row {
            for column in dimension.min_column..=dimension.max_column {
                assert_eq!(
                    reloaded.get_formatted_cell_value(sheet, row, column),
                    ordinal.get_formatted_cell_value(sheet, row, column),
                    "value at sheet {sheet} row {row} column {column}"
                );
                assert_eq!(
                    reloaded.get_cell_formula(sheet, row, column),
                    ordinal.get_cell_formula(sheet, row, column),
                    "formula at sheet {sheet} row {row} column {column}"
                );
            }
        }
    }
}
