#![allow(clippy::unwrap_used)]

use crate::model::Model;

// Excel's values are from Apache POI's DateFormatTests.xlsx (Tests!A2, A3, A5, A21) and
// DateFormatNumberTests.xlsx (Tests!A11), saved by Excel in the 1904 date system.
#[test]
fn text_reads_serials_in_the_1904_date_system() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.workbook.settings.date1904 = true;

    model._set("A1", "=TEXT(17816.607951388887,\"d-m-y\")");
    model._set("A2", "=TEXT(17816.607951388887,\"dd-mm-yy\")");
    model._set("A3", "=TEXT(17816.607951388887,\"dddd-mmmm-yyyy\")");
    model._set("A4", "=TEXT(36191.170208437499,\"d-m-y\")");
    model._set("A5", "=TEXT(1.00000478703703,\"yyyy-mm-dd hh:mm:ss\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "11-10-52");
    assert_eq!(model._get_text("A2"), "11-10-52");
    assert_eq!(model._get_text("A3"), "Saturday-October-1952");
    assert_eq!(model._get_text("A4"), "1-2-03");
    assert_eq!(model._get_text("A5"), "1904-01-02 00:00:00");
}
