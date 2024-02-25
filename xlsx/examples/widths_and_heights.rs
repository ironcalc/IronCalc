use ironcalc::{base::Model, export::save_to_xlsx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new_empty("widths-and-heights", "en", "UTC")?;
    // Cell C5
    let (sheet, row, column) = (0, 5, 3);
    // Make the first column 4 times as width
    let worksheet = model.workbook.worksheet_mut(sheet)?;
    let column_width = worksheet.get_column_width(column)? * 4.0;
    worksheet.set_column_width(column, column_width)?;

    // and the first row twice as high.
    let row_height = worksheet.row_height(row)? * 2.0;
    worksheet.set_row_height(row, row_height)?;

    // saves to disk
    save_to_xlsx(&model, "widths-and-heights.xlsx")?;
    Ok(())
}
