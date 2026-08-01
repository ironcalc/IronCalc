// Example: test formula_completion positions
use ironcalc_base::Model;

fn main() -> Result<(), String> {
    let mut model = Model::new_empty("test", "en", "UTC", "en")?;

    // Test various cursor positions with FINANCE
    let tests = vec![
        ("FINANCE", 7, "Full name, cursor at end"),        // cursor after "FINANCE"
        ("FIN", 3, "Partial name 'FIN', cursor at end"),    // cursor after "FIN"
        ("FINANCE(", 8, "After opening paren"),             // cursor after "FINANCE("
        ("F", 1, "Just 'F'"),                               // cursor after "F"
        ("FINANCE(\"AAPL\",", 16, "After first arg comma"), // cursor after 1st arg + comma
        ("FINANCE(\"AAPL\", \"price\",", 23, "After second arg comma"), // cursor after 2nd arg + comma
        ("=FINANCE", 8, "With leading ="),                  // with =
        ("=FINANCE(", 9, "With = and open paren"),
    ];

    for (input, cursor, description) in &tests {
        let ctx = model.formula_completion(0, 1, 1, input, *cursor)?;
        println!(
            "input=\"{}\" cursor={} \"{}\"",
            input, cursor, description
        );
        println!(
            "  replace_from={}, expecting={:?}",
            ctx.replace_from, ctx.expecting
        );
        println!();
    }

    Ok(())
}
