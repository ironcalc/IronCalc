use super::util::new_empty_model;

#[test]
fn shared_strings_one_reference() {
    let mut model = new_empty_model();

    // Set an arbitrary string in a cell
    model._set("A1", "Hello");
    assert_eq!(model.shared_strings.len(), 1);

    // Calling the garbage collector should not do anything
    model.garbage_collector().unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // Deleting the cell will leave the string in model.shared_strings
    model.delete_cell(0, 1, 1).unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // The garbage collector should now remove the reference
    model.garbage_collector().unwrap();
    assert_eq!(model.shared_strings.len(), 0);
}

#[test]
fn shared_strings_multiple_references() {
    let mut model = new_empty_model();

    // Set an arbitrary string in a cell
    model._set("A1", "Hello");
    model._set("A2", "Hello");
    assert_eq!(model.shared_strings.len(), 1);

    // Calling the garbage collector should not do anything
    model.garbage_collector().unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // Deleting the cell will leave the string in model.shared_strings
    model.delete_cell(0, 1, 1).unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // The garbage collector will leave the string in model.shared_strings since A2 is still referencing it
    model.garbage_collector().unwrap();
    assert_eq!(model.shared_strings.len(), 1);
}

#[test]
fn evaluate_runs_garbage_collector() {
    let mut model = new_empty_model();

    // Set an arbitrary string in a cell
    model._set("A1", "Hello");
    assert_eq!(model.shared_strings.len(), 1);

    // Deleting the cell will leave the string in model.shared_strings
    model.delete_cell(0, 1, 1).unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // The garbage collector (via model.evaluate) should now remove the reference
    model.evaluate();
    assert_eq!(model.shared_strings.len(), 0);
}
