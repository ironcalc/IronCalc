#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

fn sanitize_name(stem: &str) -> String {
    stem.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

struct FolderConfig {
    /// Directory relative to the crate root (also used in the generated test path)
    dir: &'static str,
    /// Prefix for the generated test function names
    prefix: &'static str,
    /// Whether to call test_load_and_saving in addition to test_file
    test_save: bool,
    /// File stems (without extension) to skip entirely
    skip: &'static [&'static str],
    /// Whether to recurse into subdirectories
    recursive: bool,
}

const FOLDERS: &[FolderConfig] = &[
    FolderConfig {
        dir: "tests/calc_tests",
        prefix: "test_calc",
        test_save: true,
        skip: &[],
        recursive: true,
    },
    FolderConfig {
        dir: "tests/calc_test_no_export",
        prefix: "test_no_export",
        test_save: false,
        skip: &[],
        recursive: false,
    },
    FolderConfig {
        dir: "tests/templates",
        prefix: "test_templates",
        test_save: true,
        skip: &[],
        recursive: false,
    },
    FolderConfig {
        dir: "tests/docs",
        prefix: "test_docs",
        test_save: true,
        // Volatile (date-dependent) or numerically unstable
        skip: &["DATE", "DAY", "MONTH", "YEAR", "TAN"],
        recursive: false,
    },
];

/// Collects (fn_suffix, file_path) for every .xlsx file under `current`.
/// When `recursive` is true, descends into subdirectories and emits
/// `cargo:rerun-if-changed` for each one found.
fn collect_xlsx(
    base: &Path,
    current: &Path,
    cfg_dir: &str,
    recursive: bool,
    skip: &[&str],
    results: &mut Vec<(String, String)>,
) {
    let mut entries: Vec<_> = match fs::read_dir(current) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if path.is_dir() {
            if recursive {
                println!("cargo:rerun-if-changed={}", path.display());
                collect_xlsx(base, &path, cfg_dir, recursive, skip, results);
            }
            continue;
        }

        if !file_name_str.ends_with(".xlsx") || file_name_str.starts_with('~') {
            continue;
        }

        let stem = &file_name_str[..file_name_str.len() - 5];

        if skip.contains(&stem) {
            continue;
        }

        // Build fn_suffix: subdirectory components (relative to base) + stem.
        let rel = path.strip_prefix(base).unwrap();
        let rel_dir = rel.parent().unwrap_or(Path::new(""));
        let mut parts: Vec<String> = rel_dir
            .components()
            .filter(|c| matches!(c, std::path::Component::Normal(_)))
            .map(|c| sanitize_name(&c.as_os_str().to_string_lossy()))
            .collect();
        parts.push(sanitize_name(stem));
        let fn_suffix = parts.join("_");

        let file_path = format!("{}/{}", cfg_dir, rel.to_string_lossy());
        results.push((fn_suffix, file_path));
    }
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("generated_tests.rs");

    let mut code = String::new();

    for cfg in FOLDERS {
        // Tell Cargo to re-run this script if the top-level directory changes.
        println!("cargo:rerun-if-changed={}", cfg.dir);

        let dir_path = Path::new(cfg.dir);
        let mut files = Vec::new();
        collect_xlsx(
            dir_path,
            dir_path,
            cfg.dir,
            cfg.recursive,
            cfg.skip,
            &mut files,
        );

        for (fn_suffix, file_path) in files {
            let fn_name = format!("{}_{}", cfg.prefix, fn_suffix);

            if cfg.test_save {
                writeln!(
                    code,
                    r#"#[allow(non_snake_case)]
#[test]
fn {fn_name}() {{
    let temp_folder = std::env::temp_dir();
    let dir = temp_folder.join(format!("{{}}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let result = std::panic::catch_unwind(|| {{
        ironcalc::compare::test_file("{file_path}").unwrap_or_else(|e| panic!("{{}}", e));
        ironcalc::compare::test_load_and_saving("{file_path}", &dir)
            .unwrap_or_else(|e| panic!("{{}}", e));
    }});
    std::fs::remove_dir_all(&dir).unwrap();
    result.unwrap();
}}
"#
                )
                .unwrap();
            } else {
                writeln!(
                    code,
                    r#"#[allow(non_snake_case)]
#[test]
fn {fn_name}() {{
    ironcalc::compare::test_file("{file_path}").unwrap_or_else(|e| panic!("{{}}", e));
}}
"#
                )
                .unwrap();
            }
        }
    }

    fs::write(&dest, code).expect("failed to write generated_tests.rs");
}
