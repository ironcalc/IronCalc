#![allow(clippy::unwrap_used)]

use crate::{test::util::new_empty_model, types::CellType};

#[test]
fn test_model_set_fns_related_to_merge_cells() {
    let mut model = new_empty_model();

    //creating a merge cell of D1:F2
    model.merge_cells(0, "D1:F2").unwrap();

    //Updating the mother cell of Merge cells and expecting the update to go through
    model.set_user_input(0, 1, 4, "Hello".to_string()).unwrap();
    assert_eq!(model.get_cell_content(0, 1, 4).unwrap(), "Hello");
    assert_eq!(model.get_cell_type(0, 1, 4).unwrap(), CellType::Text);

    // Updating cell which is not in Merge cell block
    assert_eq!(model.set_user_input(0, 1, 3, "Hello".to_string()), Ok(()));
    assert_eq!(model.get_cell_content(0, 1, 3), Ok("Hello".to_string()));
    assert_eq!(model.get_cell_type(0, 1, 3), Ok(CellType::Text));

    // 1: testing with set_user_input()
    assert_eq!(
        model
            .set_user_input(0, 1, 5, "Hello".to_string()),
        Err("Cell row : 1, col : 5 is part of merged cell block, so singular update to the cell is not possible".to_string())
    );
    assert_eq!(model.get_cell_content(0, 1, 5), Ok("".to_string()));
    assert_eq!(model.get_cell_type(0, 1, 5), Ok(CellType::Number));

    // 2: testing with update_cell_with_bool()
    assert_eq!(
        model
            .update_cell_with_bool(0, 1, 5, true),
        Err("Cell row : 1, col : 5 is part of merged cell block, so singular update to the cell is not possible".to_string())
    );
    assert_eq!(model.get_cell_content(0, 1, 5), Ok("".to_string()));
    assert_eq!(model.get_cell_type(0, 1, 5), Ok(CellType::Number));

    // 3: testing with update_cell_with_formula()
    assert_eq!(
        model
            .update_cell_with_formula(0, 1, 5, "=SUM(A1+A2)".to_string()),
        Err("Cell row : 1, col : 5 is part of merged cell block, so singular update to the cell is not possible".to_string())
    );
    assert_eq!(model.get_cell_type(0, 1, 5), Ok(CellType::Number));

    // 4: testing with update_cell_with_number()
    assert_eq!(
        model
            .update_cell_with_number(0, 1, 5, 10.0),
        Err("Cell row : 1, col : 5 is part of merged cell block, so singular update to the cell is not possible".to_string())
    );
    assert_eq!(model.get_cell_content(0, 1, 5), Ok("".to_string()));
    assert_eq!(model.get_cell_type(0, 1, 5), Ok(CellType::Number));

    // 5: testing with update_cell_with_text()
    assert_eq!(
        model
            .update_cell_with_text(0, 1, 5, "new text"),
        Err("Cell row : 1, col : 5 is part of merged cell block, so singular update to the cell is not possible".to_string())
    );
    assert_eq!(model.get_cell_content(0, 1, 5), Ok("".to_string()));
    assert_eq!(model.get_cell_type(0, 1, 5), Ok(CellType::Number));
}

#[test]
fn test_model_merge_cells_crud_api() {
    let mut model = new_empty_model();

    //creating a merge cell of D4:F6
    model.merge_cells(0, "D4:F6").unwrap();
    model
        .set_user_input(0, 4, 4, "Merge Block".to_string())
        .unwrap();
    // CRUD APIS testing on Merge Cells

    // Case1: Creating a new merge cell without overlapping
    // Newly created Merge block is left to D4:F6
    assert_eq!(model.merge_cells(0, "A1:B4"), Ok(()));
    assert_eq!(model.workbook.worksheet(0).unwrap().merge_cells.len(), 2);
    model.set_user_input(0, 1, 1, "left".to_string()).unwrap();

    // Newly created Merge block is right to D4:F6
    assert_eq!(model.merge_cells(0, "G1:H7"), Ok(()));
    assert_eq!(model.workbook.worksheet(0).unwrap().merge_cells.len(), 3);
    model.set_user_input(0, 1, 7, "right".to_string()).unwrap();

    // Newly created Merge block is above to D4:F6
    assert_eq!(model.merge_cells(0, "C1:D3"), Ok(()));
    assert_eq!(model.workbook.worksheet(0).unwrap().merge_cells.len(), 4);
    model.set_user_input(0, 1, 3, "top".to_string()).unwrap();

    // Newly created Merge block is down to D4:F6
    assert_eq!(model.merge_cells(0, "D8:E9"), Ok(()));
    assert_eq!(model.workbook.worksheet(0).unwrap().merge_cells.len(), 5);
    model.set_user_input(0, 8, 4, "down".to_string()).unwrap();

    //Case2: Creating a new merge cell with overlapping with other 3 merged cell
    assert_eq!(model.merge_cells(0, "C1:G4"), Ok(()));
    assert_eq!(model.workbook.worksheet(0).unwrap().merge_cells.len(), 3);
    model
        .set_user_input(0, 1, 3, "overlapped_new_merge_block".to_string())
        .unwrap();

    // Case3: Giving wrong parsing range
    assert_eq!(
        model.merge_cells(0, "C3:A1"),
        Err("Invalid parse range. Merge Mother cell always be top left cell".to_string())
    );
    assert_eq!(
        model.merge_cells(0, "CA:A1"),
        Err("Invalid range: 'CA:A1'".to_string())
    );
    assert_eq!(
        model.merge_cells(0, "C0:A1"),
        Err("Invalid range: 'C0:A1'".to_string())
    );
    assert_eq!(
        model.merge_cells(0, "C1:A0"),
        Err("Invalid range: 'C1:A0'".to_string())
    );
    assert_eq!(
        model.merge_cells(0, "C1"),
        Err("Invalid range: 'C1'".to_string())
    );
    assert_eq!(
        model.merge_cells(0, "C1:A1:B1"),
        Err("Invalid range: 'C1:A1:B1'".to_string())
    );

    // Case3: Giving wrong merge_ref, which would resulting in error (Merge cell to be deleted is not found)
    assert_eq!(
        model.unmerge_cells(0, "C1:E1"),
        Err("Invalid merge_cell_ref, Merge cell to be deleted is not found".to_string())
    );

    // Case4: unmerge scenario
    assert_eq!(model.unmerge_cells(0, "C1:G4"), Ok(()));
}
