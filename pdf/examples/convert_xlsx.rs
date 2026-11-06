use std::env;
use std::path::Path;

use ironcalc::import::load_from_xlsx;
use ironcalc_pdf::{BookmarkPolicy, ExportScope, PaperSize, PdfExportOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -p ironcalc_pdf --example convert_xlsx -- <input.xlsx> [output.pdf] [--landscape] [--no-grid] [--all-sheets]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let mut output_path = None;
    let mut landscape = false;
    let mut gridlines = true;
    let mut all_sheets = false;

    for arg in args.iter().skip(2) {
        if arg == "--landscape" {
            landscape = true;
        } else if arg == "--no-grid" {
            gridlines = false;
        } else if arg == "--all-sheets" {
            all_sheets = true;
        } else if !arg.starts_with("--") && output_path.is_none() {
            output_path = Some(arg.clone());
        }
    }

    let out_file = output_path.unwrap_or_else(|| {
        let p = Path::new(input_path);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        format!("{stem}.pdf")
    });

    println!("Loading workbook: {input_path}...");
    let model = load_from_xlsx(input_path, "en", "UTC", "en")?;

    println!("Configuring PDF export options...");
    let mut options = PdfExportOptions::default()
        .with_paper_size(PaperSize::A4)
        .fit_to_page()
        .with_centering(true, false)
        .with_gridlines(gridlines)
        .with_bookmarks(BookmarkPolicy::AlwaysBySheet);

    if landscape {
        options = options.landscape();
    } else {
        options = options.portrait();
    }

    if all_sheets {
        options = options.with_scope(ExportScope::EntireWorkbook);
    } else {
        options = options.with_scope(ExportScope::ActiveSheet);
    }

    println!("Rendering to PDF: {out_file}...");
    ironcalc_pdf::export_to_pdf_file(&model, &options, &out_file)?;

    println!("Done! Successfully generated {out_file}");
    Ok(())
}
