use ironcalc::{
    base::{expressions::utils::number_to_column, Model},
    export::save_to_xlsx,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new_empty("hello-calc.xlsx", "en", "UTC")?;
    // Adds a square of numbers in the first sheet
    for row in 1..100 {
        for column in 1..100 {
            let value = row * column;
            model.set_user_input(0, row, column, format!("{}", value))?;
        }
    }
    // Adds a new sheet
    model.add_sheet("Calculation")?;
    // column 100 is CV
    let last_column = number_to_column(100).ok_or("Invalid column number")?;
    let formula = format!("=SUM(Sheet1!A1:{}100)", last_column);
    model.set_user_input(1, 1, 1, formula)?;

    // evaluates
    model.evaluate();

    // saves to disk
    save_to_xlsx(&model, "hello-calc.xlsx")?;
    Ok(())
}
