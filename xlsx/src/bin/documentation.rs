//! Produces documentation of all the implemented IronCalc functions
//! and saves the result to functions.md
//!
//! Usage: documentation

use std::fs;

use ironcalc_base::model::Model;

fn main() {
    let mut doc = Vec::new();
    for function in Model::documentation() {
        doc.push(function.name);
    }
    let data = doc.join("\n");
    fs::write("functions.md", data).expect("Unable to write file");
}
