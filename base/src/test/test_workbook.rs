#![allow(clippy::unwrap_used)]

use crate::{test::util::new_empty_model, types::SheetInfo};

#[test]
fn workbook_worksheets_info() {
    let model = new_empty_model();
    let sheets_info = model.workbook.get_worksheets_info();
    assert_eq!(
        sheets_info[0],
        SheetInfo {
            name: "Sheet1".to_string(),
            state: "visible".to_string(),
            sheet_id: 1,
            color: None
        }
    );
}
