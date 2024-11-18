use std::io::Read;
use std::{env, fs, io};
use uuid::Uuid;

use ironcalc::compare::{test_file, test_load_and_saving};
use ironcalc::export::save_to_xlsx;
use ironcalc::import::{load_from_icalc, load_from_xlsx, load_from_xlsx_bytes};
use ironcalc_base::types::{HorizontalAlignment, VerticalAlignment};
use ironcalc_base::Model;

use std::process::{Command, ExitStatus};

#[test]
fn test_simple_csv() {
    let name = "example_csv.csv";
    let ic_name = "example_csv.ic";
    let csv_data = "1,2,3\n4,5,6";

    fs::write(name, csv_data).expect("Unable to write file");

    let status = Command::new("cargo")
        .args(["run", "--bin", "xlsx_2_icalc", "--", name])
        .status()
        .expect("failed to execute process");

    assert!(status.success());

    let model = load_from_icalc(ic_name).expect("file should exist after command is run.");

    assert_eq!(model.get_cell_content(0, 1, 1), Ok("1".to_owned()));
    assert_eq!(model.get_cell_content(0, 1, 2), Ok("2".to_owned()));
    assert_eq!(model.get_cell_content(0, 1, 3), Ok("3".to_owned()));

    assert_eq!(model.get_cell_content(0, 2, 1), Ok("4".to_owned()));
    assert_eq!(model.get_cell_content(0, 2, 2), Ok("5".to_owned()));
    assert_eq!(model.get_cell_content(0, 2, 3), Ok("6".to_owned()));

    assert!(fs::remove_file(name).is_ok());
    assert!(fs::remove_file(ic_name).is_ok());
}
