#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_empty_model() {
    let mut model = new_empty_model();
    // set a string
    model._set("A1", "Hello");
    assert_eq!(model.shared_strings.len(), 1);
    // calling the gc doesn't do anything
    model.gc().unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // If we delete the cell the string is still in the list
    model.delete_cell(0, 1, 1).unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // after the gc the string is no longer present
    model.gc().unwrap();
    assert_eq!(model.shared_strings.len(), 0);
}