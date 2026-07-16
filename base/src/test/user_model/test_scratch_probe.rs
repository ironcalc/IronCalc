#[test]
fn engine_grid_edge_displacement() {
    let mut um = crate::UserModel::from_model(crate::test::util::new_empty_model());
    um.set_user_input(0, 1, 2, "=A1048577").unwrap();
    eprintln!("out-of-grid round trip: {:?}", um.get_cell_content(0, 1, 2));
    eprintln!("out-of-grid value: {:?}", um.get_formatted_cell_value(0, 1, 2));

    let mut um2 = crate::UserModel::from_model(crate::test::util::new_empty_model());
    um2.set_user_input(0, 1, 2, "=SUM(A1:A1048576)").unwrap();
    um2.insert_rows(0, 3, 1).unwrap();
    eprintln!("explicit full col after insert: {:?}", um2.get_cell_content(0, 1, 2));
}
