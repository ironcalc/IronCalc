use super::util::new_empty_model;

#[test]
fn shared_strings() {
    let mut model = new_empty_model();

    // Set an arbitrary string in a cell
    model._set("A1", "Hello");
    model._set("A2", "Hello");
    assert_eq!(model.shared_strings.len(), 1);

    // Calling the garbage collector should not do anything since A1 and A2 reference the same string
    model.garbage_collector().unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // Deleting a cell should leave the string in model.shared_strings because the GC hasn't run yet
    model.delete_cell(0, 1, 1).unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // The garbage collector should leave the string in model.shared_strings since A2 is still referencing it
    model.garbage_collector().unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // Deleting a cell should leave the string in model.shared_strings because the GC hasn't run yet
    model.delete_cell(0, 2, 1).unwrap();
    assert_eq!(model.shared_strings.len(), 1);

    // The garbage collector should remove the reference now that A1 and A2 are gone
    model.garbage_collector().unwrap();
    assert_eq!(model.shared_strings.len(), 0);
}
