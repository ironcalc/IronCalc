#![allow(clippy::unwrap_used)]

use crate::{mock_time, test::util::new_empty_model};

// 14:44 20 Mar 2023 Berlin
const TIMESTAMP_2023: i64 = 1679319865208;

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=NOW(1)");
    model.evaluate();

    assert_eq!(
        model._get_text("A1"),
        "#ERROR!",
        "NOW should not accept arguments"
    );
}

#[test]
fn returns_date_time() {
    mock_time::set_mock_time(TIMESTAMP_2023);
    let mut model = new_empty_model();
    model._set("A1", "=NOW()");
    model.evaluate();
    let text = model._get_text("A1");
    assert_eq!(text, *"20/03/2023 13:44:25");
}
