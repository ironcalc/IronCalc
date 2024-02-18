use ironcalc::{base::model::Model, export::save_to_xlsx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new_empty("hello_styles", "en", "UTC")?;

    // We are going to change styles in cell A1
    let sheet = 0;
    let row = 1;
    let column = 1;
    let mut style = model.get_style_for_cell(sheet, row, column);
    style.fill.fg_color = Some("#FFEE11".to_string());
    style.font.b = true;
    style.font.color = Some("#EEFF22".to_string());
    model.set_cell_style(sheet, row, column, &style)?;

    // evaluates (unnecessary in this case)
    model.evaluate();

    // saves to disk
    save_to_xlsx(&model, "hello-styles.xlsx")?;
    Ok(())
}
