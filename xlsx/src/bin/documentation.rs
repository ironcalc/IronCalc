//! Produces documentation of all the implemented IronCalc functions
//! and saves the result to functions.md
//!
//! Usage: documentation

use std::fs;

use ironcalc_base::Model;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let output_file = if args.len() == 3 {
        &args[2]
    } else if args.len() == 1 {
        "functions.md"
    } else {
        panic!("Usage: {} -o <file.md>", args[0]);
    };
    let mut doc = Vec::new();
    doc.push("# List of Functions implemented in IronCalc\n\n".to_owned());
    for function in Model::documentation() {
        doc.push(format!("* {}\n", &function.name));
    }
    let data = doc.join("");
    fs::write(output_file, data).expect("Unable to write file");
}
