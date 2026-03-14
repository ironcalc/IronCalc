use ironcalc::{base::Model, export::save_to_xlsx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new_empty("widths-and-heights", "en", "UTC", "en")?;
    // Cell C5
    let (sheet, row, column) = (0, 5, 3);
    // Make the first column 4 times as width
    let column_width = model.get_column_width(sheet, column)? * 4.0;
    model.set_column_width(sheet, column, column_width)?;

    // and the first row twice as high.
    let row_height = model.get_row_height(sheet, row)? * 2.0;
    model.set_row_height(sheet, row, row_height)?;

    // saves to disk
    save_to_xlsx(&model, "widths-and-heights.xlsx")?;
    Ok(())
}
