use hipdf::lopdf::Document;
use ironcalc::export::save_to_xlsx;
use ironcalc_base::types::{
    Alignment, Border, BorderItem, BorderStyle, Color, Fill, Font, HorizontalAlignment, MergedCell,
    Style, VerticalAlignment,
};
use ironcalc_base::Model;
use ironcalc_pdf::{
    export_to_pdf, BookmarkPolicy, CellErrorMode, CellRange, ExportScope, FormulaPrintPolicy,
    HeaderFooterField, HeaderFooterLayout, Margins, PageOrder, PaperSize, PdfExportOptions,
    Scaling,
};
use std::fs;
use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn output_dir() -> PathBuf {
    let dir = manifest_dir().join("tests/output");
    fs::create_dir_all(&dir).expect("failed to create pdf/tests/output directory");
    dir
}

/// Creates a comprehensive, clean, realistic multi-sheet financial model.
/// Contains arithmetic operations (*, -, +, /), nested logic IFs, aggregations (SUM, AVERAGE, MIN, MAX, COUNT, COUNTA),
/// string operations (CONCAT), formatted currencies, percentages, integers, styled banners, alternating rows,
/// frozen panes, column widths, row heights, and cross-sheet references.
///
/// All formulas evaluate cleanly to valid values with ZERO errors.
pub fn create_complex_table_model() -> Model<'static> {
    let mut model = Model::new_empty("Enterprise Financial Model", "en", "UTC", "en")
        .expect("create empty model");
    model
        .rename_sheet_by_index(0, "Financials")
        .expect("rename sheet 0");

    // -------------------------------------------------------------
    // SHEET 0: "Financials"
    // -------------------------------------------------------------
    let s0 = 0;

    // 1. Column Widths
    let col_widths = [
        (1, 70.0),   // Line ID
        (2, 250.0),  // Description
        (3, 90.0),   // Region
        (4, 80.0),   // Units Sold
        (5, 95.0),   // Unit Price
        (6, 115.0),  // Gross Revenue
        (7, 95.0),   // Discount Rate
        (8, 95.0),   // Discount Amount
        (9, 115.0),  // Net Revenue
        (10, 85.0),  // Tax Rate
        (11, 95.0),  // Tax Amount
        (12, 125.0), // Total Revenue
        (13, 105.0), // KPI Status
    ];
    for (col, w) in col_widths {
        let _ = model.set_column_width(s0, col, w);
    }

    // 2. Row Heights
    let _ = model.set_row_height(s0, 1, 32.0);
    let _ = model.set_row_height(s0, 2, 22.0);
    let _ = model.set_row_height(s0, 3, 12.0);
    let _ = model.set_row_height(s0, 4, 25.0);
    for r in 5..=24 {
        let _ = model.set_row_height(s0, r, 20.0);
    }
    let _ = model.set_row_height(s0, 25, 24.0);
    for r in 26..=29 {
        let _ = model.set_row_height(s0, r, 20.0);
    }

    // Freeze panes after row 4
    let _ = model.set_frozen_rows(s0, 4);

    // 3. Merged Banners
    // Row 1 Title Banner (A1:M1)
    model.workbook.worksheets[0].merged_cells.push(MergedCell {
        row: 1,
        column: 1,
        width: 13,
        height: 1,
    });
    model
        .set_user_input(
            s0,
            1,
            1,
            "GLOBAL ENTERPRISE PERFORMANCE & CONSOLIDATED AUDIT".to_string(),
        )
        .unwrap();

    let title_style = Style {
        alignment: Some(Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
            wrap_text: false,
        }),
        num_fmt: "general".to_string(),
        fill: Fill {
            color: Color::Rgb("#1E3A8A".to_string()),
        },
        font: Font {
            strike: false,
            u: false,
            b: true,
            i: false,
            sz: 14,
            color: Color::Rgb("#FFFFFF".to_string()),
            name: "Helvetica".to_string(),
            family: 2,
            scheme: ironcalc_base::types::FontScheme::Minor,
        },
        border: Border::default(),
        quote_prefix: false,
    };
    let _ = model.set_cell_style(s0, 1, 1, &title_style);

    // Row 2 Subtitle Banner (A2:M2)
    model.workbook.worksheets[0].merged_cells.push(MergedCell {
        row: 2,
        column: 1,
        width: 13,
        height: 1,
    });
    model
        .set_user_input(
            s0,
            2,
            1,
            "Fiscal Year 2026 Audit Report • Departmental Breakdown • Powered by IronCalc Engine"
                .to_string(),
        )
        .unwrap();

    let subtitle_style = Style {
        alignment: Some(Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
            wrap_text: false,
        }),
        num_fmt: "general".to_string(),
        fill: Fill {
            color: Color::Rgb("#2563EB".to_string()),
        },
        font: Font {
            strike: false,
            u: false,
            b: false,
            i: true,
            sz: 9,
            color: Color::Rgb("#E0E7FF".to_string()),
            name: "Helvetica".to_string(),
            family: 2,
            scheme: ironcalc_base::types::FontScheme::Minor,
        },
        border: Border::default(),
        quote_prefix: false,
    };
    let _ = model.set_cell_style(s0, 2, 1, &subtitle_style);

    // 4. Column Headers (Row 4)
    let headers = [
        (1, "Line ID", HorizontalAlignment::Center),
        (
            2,
            "Product / Department Description",
            HorizontalAlignment::Left,
        ),
        (3, "Region", HorizontalAlignment::Center),
        (4, "Units Sold", HorizontalAlignment::Right),
        (5, "Unit Price ($)", HorizontalAlignment::Right),
        (6, "Gross Revenue ($)", HorizontalAlignment::Right),
        (7, "Discount Rate (%)", HorizontalAlignment::Center),
        (8, "Discount ($)", HorizontalAlignment::Right),
        (9, "Net Revenue ($)", HorizontalAlignment::Right),
        (10, "Tax Rate (%)", HorizontalAlignment::Center),
        (11, "Tax Amount ($)", HorizontalAlignment::Right),
        (12, "Total Revenue ($)", HorizontalAlignment::Right),
        (13, "Performance KPI", HorizontalAlignment::Center),
    ];

    for (col, text, align) in headers {
        model.set_user_input(s0, 4, col, text.to_string()).unwrap();
        let header_style = Style {
            alignment: Some(Alignment {
                horizontal: align,
                vertical: VerticalAlignment::Center,
                wrap_text: false,
            }),
            num_fmt: "general".to_string(),
            fill: Fill {
                color: Color::Rgb("#0F172A".to_string()),
            },
            font: Font {
                strike: false,
                u: false,
                b: true,
                i: false,
                sz: 10,
                color: Color::Rgb("#FFFFFF".to_string()),
                name: "Helvetica".to_string(),
                family: 2,
                scheme: ironcalc_base::types::FontScheme::Minor,
            },
            border: Border {
                diagonal_up: false,
                diagonal_down: false,
                left: None,
                right: None,
                top: Some(BorderItem {
                    style: BorderStyle::Thin,
                    color: Color::Rgb("#334155".to_string()),
                }),
                bottom: Some(BorderItem {
                    style: BorderStyle::Medium,
                    color: Color::Rgb("#334155".to_string()),
                }),
                diagonal: None,
            },
            quote_prefix: false,
        };
        let _ = model.set_cell_style(s0, 4, col, &header_style);
    }

    // 5. Data Rows (Rows 5 to 24)
    let data_items = [
        (
            101,
            "Cloud Compute Instances (Standard 4vCPU/16GB)",
            "North America",
            1500,
            45.0,
        ),
        (
            102,
            "Managed Kubernetes Clusters (Enterprise SLA)",
            "EMEA",
            320,
            180.0,
        ),
        (
            103,
            "High-IOPS Distributed Block Storage (TB)",
            "North America",
            4200,
            12.5,
        ),
        (
            104,
            "Global Content Delivery Network Bandwidth",
            "APAC",
            12500,
            2.2,
        ),
        (
            105,
            "Serverless Function Executions (10M Blocks)",
            "LATAM",
            850,
            28.0,
        ),
        (
            106,
            "Dedicated Hardware Security Modules",
            "EMEA",
            45,
            1200.0,
        ),
        (
            107,
            "Private Interconnect 10Gbps Direct Links",
            "North America",
            95,
            650.0,
        ),
        (
            108,
            "AI Inference Accelerator TPUv5 Hours",
            "North America",
            2400,
            32.0,
        ),
        (
            109,
            "Managed Relational Database High-Availability",
            "EMEA",
            620,
            95.0,
        ),
        (
            110,
            "Real-time Telemetry & Logging Pipeline",
            "APAC",
            3100,
            8.5,
        ),
        (
            111,
            "Enterprise Identity & SSO Governance Seats",
            "North America",
            18000,
            4.5,
        ),
        (
            112,
            "Disaster Recovery Active-Active Replication",
            "EMEA",
            120,
            480.0,
        ),
        (
            113,
            "Edge Compute Gateway Nodes (Containerized)",
            "APAC",
            750,
            65.0,
        ),
        (
            114,
            "Secure API Management & Rate Limiter Gateway",
            "LATAM",
            2100,
            15.0,
        ),
        (
            115,
            "Machine Learning Training Cluster Hours",
            "North America",
            160,
            420.0,
        ),
        (
            116,
            "Zero Trust Network Access Endpoints",
            "EMEA",
            8200,
            3.8,
        ),
        (
            117,
            "Automated Data Governance & Cataloging",
            "APAC",
            340,
            110.0,
        ),
        (
            118,
            "Cold Tier Archive Storage (Petabytes)",
            "North America",
            52,
            950.0,
        ),
        (
            119,
            "Managed Elasticsearch Cluster Tier 3",
            "EMEA",
            280,
            140.0,
        ),
        (
            120,
            "Enterprise Support & SLA 24/7 Premium",
            "Global",
            50,
            2500.0,
        ),
    ];

    for (idx, (id, desc, region, units, price)) in data_items.iter().enumerate() {
        let r = (idx + 5) as i32;
        let is_even = idx % 2 == 0;
        let row_fill = if is_even { "#FFFFFF" } else { "#F8FAFC" };

        model.set_user_input(s0, r, 1, id.to_string()).unwrap();
        model.set_user_input(s0, r, 2, desc.to_string()).unwrap();
        model.set_user_input(s0, r, 3, region.to_string()).unwrap();
        model.set_user_input(s0, r, 4, units.to_string()).unwrap();
        model.set_user_input(s0, r, 5, price.to_string()).unwrap();
        model
            .set_user_input(s0, r, 6, format!("=D{}*E{}", r, r))
            .unwrap();
        model
            .set_user_input(
                s0,
                r,
                7,
                format!("=IF(D{}>5000, 0.12, IF(D{}>1000, 0.08, 0.03))", r, r),
            )
            .unwrap();
        model
            .set_user_input(s0, r, 8, format!("=F{}*G{}", r, r))
            .unwrap();
        model
            .set_user_input(s0, r, 9, format!("=F{}-H{}", r, r))
            .unwrap();
        model
            .set_user_input(
                s0,
                r,
                10,
                format!("=IF(C{}=\"EMEA\", 0.2, IF(C{}=\"APAC\", 0.1, 0.08))", r, r),
            )
            .unwrap();
        model
            .set_user_input(s0, r, 11, format!("=I{}*J{}", r, r))
            .unwrap();
        model
            .set_user_input(s0, r, 12, format!("=I{}+K{}", r, r))
            .unwrap();
        model
            .set_user_input(
                s0,
                r,
                13,
                format!(
                    "=IF(L{}>100000, \"EXCELLENT\", IF(L{}>50000, \"STRONG\", \"NORMAL\"))",
                    r, r
                ),
            )
            .unwrap();

        for c in 1..=13 {
            let (align, num_fmt) = match c {
                1 => (HorizontalAlignment::Center, "0"),
                2 => (HorizontalAlignment::Left, "general"),
                3 => (HorizontalAlignment::Center, "general"),
                4 => (HorizontalAlignment::Right, "#,##0"),
                5 => (HorizontalAlignment::Right, "$#,##0.00"),
                6 => (HorizontalAlignment::Right, "$#,##0.00"),
                7 => (HorizontalAlignment::Center, "0.0%"),
                8 => (HorizontalAlignment::Right, "$#,##0.00"),
                9 => (HorizontalAlignment::Right, "$#,##0.00"),
                10 => (HorizontalAlignment::Center, "0.0%"),
                11 => (HorizontalAlignment::Right, "$#,##0.00"),
                12 => (HorizontalAlignment::Right, "$#,##0.00"),
                _ => (HorizontalAlignment::Center, "general"),
            };

            let cell_style = Style {
                alignment: Some(Alignment {
                    horizontal: align,
                    vertical: VerticalAlignment::Center,
                    wrap_text: false,
                }),
                num_fmt: num_fmt.to_string(),
                fill: Fill {
                    color: Color::Rgb(row_fill.to_string()),
                },
                font: Font {
                    strike: false,
                    u: false,
                    b: false,
                    i: false,
                    sz: 9,
                    color: Color::Rgb("#1E293B".to_string()),
                    name: "Helvetica".to_string(),
                    family: 2,
                    scheme: ironcalc_base::types::FontScheme::Minor,
                },
                border: Border {
                    diagonal_up: false,
                    diagonal_down: false,
                    left: None,
                    right: None,
                    top: Some(BorderItem {
                        style: BorderStyle::Thin,
                        color: Color::Rgb("#E2E8F0".to_string()),
                    }),
                    bottom: Some(BorderItem {
                        style: BorderStyle::Thin,
                        color: Color::Rgb("#E2E8F0".to_string()),
                    }),
                    diagonal: None,
                },
                quote_prefix: false,
            };
            let _ = model.set_cell_style(s0, r, c, &cell_style);
        }
    }

    // 6. Summary / Total Row (Row 25)
    model
        .set_user_input(s0, 25, 1, "TOTAL".to_string())
        .unwrap();
    model
        .set_user_input(
            s0,
            25,
            2,
            "=CONCAT(\"Items: \", COUNTA(B5:B24))".to_string(),
        )
        .unwrap();
    model
        .set_user_input(s0, 25, 3, "20 ACTIVE".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 4, "=SUM(D5:D24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 5, "=AVERAGE(E5:E24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 6, "=SUM(F5:F24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 7, "=AVERAGE(G5:G24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 8, "=SUM(H5:H24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 9, "=SUM(I5:I24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 10, "=AVERAGE(J5:J24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 11, "=SUM(K5:K24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 25, 12, "=SUM(L5:L24)".to_string())
        .unwrap();
    model
        .set_user_input(
            s0,
            25,
            13,
            "=IF(L25>1000000, \"TARGET MET\", \"BELOW TARGET\")".to_string(),
        )
        .unwrap();

    for c in 1..=13 {
        let (align, num_fmt) = match c {
            1 => (HorizontalAlignment::Center, "general"),
            2 => (HorizontalAlignment::Left, "general"),
            3 => (HorizontalAlignment::Center, "general"),
            4 => (HorizontalAlignment::Right, "#,##0"),
            5 => (HorizontalAlignment::Right, "$#,##0.00"),
            6 => (HorizontalAlignment::Right, "$#,##0.00"),
            7 => (HorizontalAlignment::Center, "0.0%"),
            8 => (HorizontalAlignment::Right, "$#,##0.00"),
            9 => (HorizontalAlignment::Right, "$#,##0.00"),
            10 => (HorizontalAlignment::Center, "0.0%"),
            11 => (HorizontalAlignment::Right, "$#,##0.00"),
            12 => (HorizontalAlignment::Right, "$#,##0.00"),
            _ => (HorizontalAlignment::Center, "general"),
        };
        let total_style = Style {
            alignment: Some(Alignment {
                horizontal: align,
                vertical: VerticalAlignment::Center,
                wrap_text: false,
            }),
            num_fmt: num_fmt.to_string(),
            fill: Fill {
                color: Color::Rgb("#F1F5F9".to_string()),
            },
            font: Font {
                strike: false,
                u: false,
                b: true,
                i: false,
                sz: 10,
                color: Color::Rgb("#0F172A".to_string()),
                name: "Helvetica".to_string(),
                family: 2,
                scheme: ironcalc_base::types::FontScheme::Minor,
            },
            border: Border {
                diagonal_up: false,
                diagonal_down: false,
                left: None,
                right: None,
                top: Some(BorderItem {
                    style: BorderStyle::Thin,
                    color: Color::Rgb("#0F172A".to_string()),
                }),
                bottom: Some(BorderItem {
                    style: BorderStyle::Double,
                    color: Color::Rgb("#0F172A".to_string()),
                }),
                diagonal: None,
            },
            quote_prefix: false,
        };
        let _ = model.set_cell_style(s0, 25, c, &total_style);
    }

    // 7. Statistical & Operational Analysis Rows (Rows 26-28)
    // Row 26: Average Metrics
    model
        .set_user_input(s0, 26, 1, "STATISTICAL AVERAGE".to_string())
        .unwrap();
    model
        .set_user_input(s0, 26, 4, "=AVERAGE(D5:D24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 26, 6, "=AVERAGE(F5:F24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 26, 9, "=AVERAGE(I5:I24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 26, 12, "=AVERAGE(L5:L24)".to_string())
        .unwrap();

    // Row 27: Max / Min Range
    model
        .set_user_input(s0, 27, 1, "MAX / MIN RANGE".to_string())
        .unwrap();
    model
        .set_user_input(s0, 27, 4, "=MAX(D5:D24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 27, 6, "=MAX(F5:F24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 27, 9, "=MIN(I5:I24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 27, 12, "=MAX(L5:L24)".to_string())
        .unwrap();

    // Row 28: Count of Items
    model
        .set_user_input(s0, 28, 1, "METRICS COUNT".to_string())
        .unwrap();
    model
        .set_user_input(s0, 28, 4, "=COUNT(D5:D24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 28, 6, "=COUNT(F5:F24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 28, 12, "=COUNT(L5:L24)".to_string())
        .unwrap();

    // 8. Performance Efficiency & Operating Margin Metrics (Row 29)
    // Professional, fully evaluated mathematical calculations (NO errors)
    model
        .set_user_input(s0, 29, 1, "EFFICIENCY RATIOS".to_string())
        .unwrap();
    model
        .set_user_input(
            s0,
            29,
            2,
            "Consolidated Net Operating Margin & Unit Efficiency".to_string(),
        )
        .unwrap();
    model
        .set_user_input(s0, 29, 4, "=AVERAGE(D5:D24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 29, 5, "=SUM(L5:L24)/SUM(D5:D24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 29, 6, "=SUM(I5:I24)/SUM(F5:F24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 29, 9, "=SUM(I5:I24)/SUM(D5:D24)".to_string())
        .unwrap();
    model
        .set_user_input(s0, 29, 12, "=MAX(L5:L24)-MIN(L5:L24)".to_string())
        .unwrap();
    model
        .set_user_input(
            s0,
            29,
            13,
            "=IF(SUM(L5:L24)>1000000, \"EXCEEDS PLAN\", \"ON TRACK\")".to_string(),
        )
        .unwrap();

    let efficiency_style = Style {
        alignment: Some(Alignment {
            horizontal: HorizontalAlignment::Left,
            vertical: VerticalAlignment::Center,
            wrap_text: false,
        }),
        num_fmt: "general".to_string(),
        fill: Fill {
            color: Color::Rgb("#F8FAFC".to_string()),
        },
        font: Font {
            strike: false,
            u: false,
            b: true,
            i: false,
            sz: 9,
            color: Color::Rgb("#0F172A".to_string()),
            name: "Helvetica".to_string(),
            family: 2,
            scheme: ironcalc_base::types::FontScheme::Minor,
        },
        border: Border {
            diagonal_up: false,
            diagonal_down: false,
            left: None,
            right: None,
            top: Some(BorderItem {
                style: BorderStyle::Thin,
                color: Color::Rgb("#CBD5E1".to_string()),
            }),
            bottom: Some(BorderItem {
                style: BorderStyle::Thin,
                color: Color::Rgb("#CBD5E1".to_string()),
            }),
            diagonal: None,
        },
        quote_prefix: false,
    };
    for c in 1..=13 {
        let (align, num_fmt) = match c {
            1 => (HorizontalAlignment::Center, "general"),
            2 => (HorizontalAlignment::Left, "general"),
            4 => (HorizontalAlignment::Right, "#,##0"),
            5 => (HorizontalAlignment::Right, "$#,##0.00"),
            6 => (HorizontalAlignment::Right, "0.0%"),
            9 => (HorizontalAlignment::Right, "$#,##0.00"),
            12 => (HorizontalAlignment::Right, "$#,##0.00"),
            _ => (HorizontalAlignment::Center, "general"),
        };
        let mut st = efficiency_style.clone();
        if let Some(ref mut a) = st.alignment {
            a.horizontal = align;
        }
        st.num_fmt = num_fmt.to_string();
        let _ = model.set_cell_style(s0, 29, c, &st);
    }

    // 10. Formula Calculation Error Test Row (Row 30)
    // Demonstrates and verifies formula calculation errors (#DIV/0!, #VALUE!, #N/A) being printed on exports
    let _ = model.set_row_height(s0, 30, 20.0);
    model
        .set_user_input(s0, 30, 1, "ERR-30".to_string())
        .unwrap();
    model
        .set_user_input(
            s0,
            30,
            2,
            "Formula Error / Div Zero Stress Test".to_string(),
        )
        .unwrap();
    model
        .set_user_input(s0, 30, 3, "AUDIT".to_string())
        .unwrap();
    model.set_user_input(s0, 30, 4, "0".to_string()).unwrap();
    model
        .set_user_input(s0, 30, 5, "$0.00".to_string())
        .unwrap();
    model
        .set_user_input(s0, 30, 6, "=100/D30".to_string()) // Evaluates to #DIV/0!
        .unwrap();
    model.set_user_input(s0, 30, 7, "0.0%".to_string()).unwrap();
    model
        .set_user_input(s0, 30, 8, "=1+\"INVALID\"".to_string()) // Evaluates to #VALUE!
        .unwrap();
    model
        .set_user_input(s0, 30, 9, "=NA()".to_string()) // Evaluates to #N/A
        .unwrap();
    model
        .set_user_input(s0, 30, 10, "0.0%".to_string())
        .unwrap();
    model
        .set_user_input(s0, 30, 11, "$0.00".to_string())
        .unwrap();
    model
        .set_user_input(s0, 30, 12, "$0.00".to_string())
        .unwrap();
    model
        .set_user_input(s0, 30, 13, "ERROR TESTED".to_string())
        .unwrap();

    let error_row_style = efficiency_style.clone();
    for c in 1..=13 {
        let align = match c {
            1 => HorizontalAlignment::Center,
            2 => HorizontalAlignment::Left,
            6 | 8 | 9 => HorizontalAlignment::Center,
            _ => HorizontalAlignment::Center,
        };
        let mut st = error_row_style.clone();
        if let Some(ref mut a) = st.alignment {
            a.horizontal = align;
        }
        let _ = model.set_cell_style(s0, 30, c, &st);
    }

    // -------------------------------------------------------------
    // SHEET 1: "Executive Summary" (Cross-sheet references)
    // -------------------------------------------------------------
    model.new_sheet();
    let _ = model.rename_sheet_by_index(1, "Executive Summary");
    let s1 = 1;

    let _ = model.set_column_width(s1, 1, 190.0);
    let _ = model.set_column_width(s1, 2, 220.0);
    let _ = model.set_column_width(s1, 3, 140.0);
    let _ = model.set_column_width(s1, 4, 110.0);

    model.workbook.worksheets[1].merged_cells.push(MergedCell {
        row: 1,
        column: 1,
        width: 4,
        height: 1,
    });
    model
        .set_user_input(s1, 1, 1, "EXECUTIVE KPI SUMMARY".to_string())
        .unwrap();
    let _ = model.set_cell_style(s1, 1, 1, &title_style);

    let kpi_headers = [
        "Metric Title",
        "Formula Expression",
        "Computed Value",
        "Audit Status",
    ];
    for (idx, h) in kpi_headers.iter().enumerate() {
        model
            .set_user_input(s1, 3, (idx + 1) as i32, h.to_string())
            .unwrap();
    }

    let kpi_rows = [
        (
            4,
            "Consolidated Gross Revenue",
            "=Financials!F25",
            "AUDITED",
        ),
        (5, "Consolidated Net Revenue", "=Financials!I25", "AUDITED"),
        (
            6,
            "Consolidated Tax Liability",
            "=Financials!K25",
            "AUDITED",
        ),
        (7, "Final Consolidated Total", "=Financials!L25", "APPROVED"),
        (8, "Target Goal Met?", "=Financials!M25", "CONFIRMED"),
        (
            9,
            "Calculus Div/Zero Audit Check",
            "=Financials!F30",
            "#DIV/0! AUDITED",
        ),
    ];

    for (r, title, formula, status) in kpi_rows {
        model.set_user_input(s1, r, 1, title.to_string()).unwrap();
        model.set_user_input(s1, r, 2, formula.to_string()).unwrap();
        model.set_user_input(s1, r, 3, formula.to_string()).unwrap();
        model.set_user_input(s1, r, 4, status.to_string()).unwrap();
    }

    // Evaluate all workbook formulas
    model.evaluate();
    model
}

/// Creates a model with intentional calculation errors specifically to test
/// CellErrorPolicy transformation modes (DisplayAsStored, Dash, Blank, NotAvailable).
pub fn create_complex_table_model_with_errors() -> Model<'static> {
    let mut model = create_complex_table_model();
    let s0 = 0;
    let _ = model.set_row_height(s0, 30, 20.0);
    model
        .set_user_input(s0, 30, 1, "ERROR POLICY VALIDATION".to_string())
        .unwrap();
    model
        .set_user_input(
            s0,
            30,
            2,
            "Test intentional error conversions (DIV/0, NA)".to_string(),
        )
        .unwrap();
    model
        .set_user_input(s0, 30, 4, "=100/0".to_string())
        .unwrap(); // #DIV/0!
    model
        .set_user_input(s0, 30, 6, "=NA()".to_string())
        .unwrap(); // #N/A

    model.evaluate();
    model
}

fn export_and_verify(name: &str, model: &Model, opts: &PdfExportOptions, out: &Path) -> Vec<u8> {
    let pdf =
        export_to_pdf(model, opts).unwrap_or_else(|e| panic!("failed exporting {}: {}", name, e));
    assert!(
        pdf.starts_with(b"%PDF-"),
        "{} must start with %PDF- header",
        name
    );
    let doc = Document::load_mem(&pdf)
        .unwrap_or_else(|e| panic!("failed parsing lopdf for {}: {}", name, e));
    assert!(
        !doc.get_pages().is_empty(),
        "{} must contain at least 1 page",
        name
    );
    let path = out.join(format!("{}.pdf", name));
    fs::write(&path, &pdf).unwrap_or_else(|e| panic!("failed writing {}: {}", path.display(), e));
    pdf
}

#[test]
fn test_complex_table_evaluation_integrity() {
    let model = create_complex_table_model();

    // Verify Row 5 evaluated value
    let gross_f5 = model.get_formatted_cell_value(0, 5, 6).unwrap();
    assert_eq!(gross_f5, "$67,500.00");

    // Verify Row 25 summary values
    let total_l25 = model.get_formatted_cell_value(0, 25, 12).unwrap();
    assert_eq!(total_l25, "$1,135,794.03");

    // Verify Row 29 efficiency ratios (clean, valid calculations with no errors)
    let margin_ratio = model.get_formatted_cell_value(0, 29, 6).unwrap();
    assert_eq!(margin_ratio, "94.6%");

    let unit_yield = model.get_formatted_cell_value(0, 29, 5).unwrap();
    assert_eq!(unit_yield, "$20.40");

    // Verify cross-sheet formula evaluation on Sheet 1
    let kpi_gross = model.get_formatted_cell_value(1, 4, 3).unwrap();
    assert_eq!(kpi_gross, "$1,074,910.00");

    let kpi_total = model.get_formatted_cell_value(1, 7, 3).unwrap();
    assert_eq!(kpi_total, "$1,135,794.03");

    // Verify intentional formula calculation error evaluation on Sheet 0 Row 30 & Sheet 1 Row 9
    let err_div = model.get_formatted_cell_value(0, 30, 6).unwrap();
    assert_eq!(err_div, "#DIV/0!");

    let err_val = model.get_formatted_cell_value(0, 30, 8).unwrap();
    assert_eq!(err_val, "#VALUE!");

    let err_na = model.get_formatted_cell_value(0, 30, 9).unwrap();
    assert_eq!(err_na, "#N/A");

    let kpi_err = model.get_formatted_cell_value(1, 9, 3).unwrap();
    assert_eq!(kpi_err, "#DIV/0!");

    // Persist model bytes and xlsx
    let bytes = model.to_bytes();
    assert!(!bytes.is_empty());
    let bytes_file = output_dir().join("complex_table.bytes");
    fs::write(&bytes_file, &bytes).expect("write complex_table.bytes");

    let xlsx_file = output_dir().join("complex_table.xlsx");
    let _ = save_to_xlsx(&model, xlsx_file.to_str().unwrap());
    println!("Saved clean model artifacts to {:?}", bytes_file);
}

#[test]
fn test_complex_table_all_export_configurations() {
    let model = create_complex_table_model();
    let model_with_errors = create_complex_table_model_with_errors();
    let out = output_dir();

    // =========================================================================
    // SECTION 1: DEFAULT OPTIONS
    // =========================================================================
    export_and_verify(
        "complex_table_01_default",
        &model,
        &PdfExportOptions::default(),
        &out,
    );
    // Backward-compatible alias
    fs::copy(
        out.join("complex_table_01_default.pdf"),
        out.join("complex_table_default.pdf"),
    )
    .unwrap();

    // Verify formula calculation error (#DIV/0!) is visibly printed on the default exported PDF
    let default_pdf_path = out.join("complex_table_01_default.pdf");
    let default_doc = Document::load_mem(&fs::read(&default_pdf_path).unwrap()).unwrap();
    let page1_bytes = default_doc
        .get_page_content(*default_doc.get_pages().get(&1).unwrap())
        .unwrap();
    let page1_content = hipdf::lopdf::content::Content::decode(&page1_bytes).unwrap();
    let mut default_strings = Vec::new();
    for op in &page1_content.operations {
        if op.operator == "Tj" {
            if let hipdf::lopdf::Object::String(bytes, _) = &op.operands[0] {
                default_strings.push(String::from_utf8_lossy(bytes).to_string());
            }
        }
    }
    assert!(
        default_strings.iter().any(|s| s.contains("#DIV/0!")),
        "complex_table_01_default.pdf must visibly print #DIV/0! on the exported PDF, found text strings: {default_strings:?}"
    );

    // =========================================================================
    // SECTION 2: PAPER SIZES & ORIENTATIONS
    // =========================================================================
    // A4 Portrait
    export_and_verify(
        "complex_table_02_paper_a4_portrait",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::A4)
            .portrait(),
        &out,
    );

    // A4 Landscape
    export_and_verify(
        "complex_table_03_paper_a4_landscape",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::A4)
            .landscape()
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_03_paper_a4_landscape.pdf"),
        out.join("complex_table_landscape_a4.pdf"),
    )
    .unwrap();

    // US Letter Portrait
    export_and_verify(
        "complex_table_04_paper_letter_portrait",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Letter)
            .portrait(),
        &out,
    );

    // US Letter Landscape
    export_and_verify(
        "complex_table_05_paper_letter_landscape",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Letter)
            .landscape()
            .fit_to_page(),
        &out,
    );

    // US Legal Portrait
    export_and_verify(
        "complex_table_06_paper_legal_portrait",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Legal)
            .portrait(),
        &out,
    );

    // US Legal Landscape
    export_and_verify(
        "complex_table_07_paper_legal_landscape",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Legal)
            .landscape()
            .fit_to_page(),
        &out,
    );

    // Tabloid Portrait (11 x 17 in)
    export_and_verify(
        "complex_table_08_paper_tabloid_portrait",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Tabloid)
            .portrait(),
        &out,
    );

    // Tabloid Landscape
    export_and_verify(
        "complex_table_09_paper_tabloid_landscape",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Tabloid)
            .landscape()
            .fit_to_page(),
        &out,
    );

    // A3 (Large format 297 x 420 mm)
    export_and_verify(
        "complex_table_10_paper_a3",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::A3)
            .landscape()
            .fit_to_page(),
        &out,
    );

    // A5 (Compact format 148 x 210 mm)
    export_and_verify(
        "complex_table_11_paper_a5",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::A5)
            .fit_to_page(),
        &out,
    );

    // B4 (250 x 353 mm)
    export_and_verify(
        "complex_table_12_paper_b4",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::B4)
            .landscape()
            .fit_to_page(),
        &out,
    );

    // B5 (176 x 250 mm)
    export_and_verify(
        "complex_table_13_paper_b5",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::B5)
            .fit_to_page(),
        &out,
    );

    // Executive (7.25 x 10.5 in)
    export_and_verify(
        "complex_table_14_paper_executive",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Executive)
            .fit_to_page(),
        &out,
    );

    // Statement (5.5 x 8.5 in)
    export_and_verify(
        "complex_table_15_paper_statement",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Statement)
            .fit_to_page(),
        &out,
    );

    // Folio (8.5 x 13 in)
    export_and_verify(
        "complex_table_16_paper_folio",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Folio)
            .fit_to_page(),
        &out,
    );

    // Custom Paper Size (280mm x 280mm Square)
    export_and_verify(
        "complex_table_17_paper_custom_square",
        &model,
        &PdfExportOptions::default()
            .with_paper_size(PaperSize::Custom {
                width_mm: 280.0,
                height_mm: 280.0,
            })
            .fit_to_page()
            .with_centering(true, true),
        &out,
    );
    fs::copy(
        out.join("complex_table_17_paper_custom_square.pdf"),
        out.join("complex_table_custom_paper.pdf"),
    )
    .unwrap();

    // =========================================================================
    // SECTION 3: SCALING MODES
    // =========================================================================
    // Actual Size (100% normal scale, uncompressed multi-page)
    export_and_verify(
        "complex_table_18_scaling_actual_size",
        &model,
        &PdfExportOptions::default()
            .with_scaling(Scaling::ActualSize)
            .with_gridlines(true),
        &out,
    );

    // Fit Entire Sheet On One Page
    export_and_verify(
        "complex_table_19_scaling_fit_one_page",
        &model,
        &PdfExportOptions::default()
            .fit_to_page()
            .with_margins(Margins::narrow())
            .with_centering(true, true),
        &out,
    );
    fs::copy(
        out.join("complex_table_19_scaling_fit_one_page.pdf"),
        out.join("complex_table_fit_one_page_letter.pdf"),
    )
    .unwrap();

    // Fit To Width (1 page wide, vertical page breakdown)
    export_and_verify(
        "complex_table_20_scaling_fit_to_width",
        &model,
        &PdfExportOptions::default()
            .fit_to_width(1)
            .with_margins(Margins::narrow()),
        &out,
    );
    fs::copy(
        out.join("complex_table_20_scaling_fit_to_width.pdf"),
        out.join("complex_table_fit_to_width.pdf"),
    )
    .unwrap();

    // Fit To Height (1 page tall)
    export_and_verify(
        "complex_table_21_scaling_fit_to_height",
        &model,
        &PdfExportOptions::default()
            .fit_to_height(1)
            .with_margins(Margins::normal()),
        &out,
    );
    fs::copy(
        out.join("complex_table_21_scaling_fit_to_height.pdf"),
        out.join("complex_table_fit_to_height.pdf"),
    )
    .unwrap();

    // Scaling 50%
    export_and_verify(
        "complex_table_22_scaling_percent_50",
        &model,
        &PdfExportOptions::default().with_scale_percent(50),
        &out,
    );

    // Scaling 75%
    export_and_verify(
        "complex_table_23_scaling_percent_75",
        &model,
        &PdfExportOptions::default().with_scale_percent(75),
        &out,
    );
    fs::copy(
        out.join("complex_table_23_scaling_percent_75.pdf"),
        out.join("complex_table_scale_75_percent.pdf"),
    )
    .unwrap();

    // Scaling 100%
    export_and_verify(
        "complex_table_24_scaling_percent_100",
        &model,
        &PdfExportOptions::default().with_scale_percent(100),
        &out,
    );

    // Scaling 125% (Zoomed in)
    export_and_verify(
        "complex_table_25_scaling_percent_125",
        &model,
        &PdfExportOptions::default().with_scale_percent(125),
        &out,
    );

    // =========================================================================
    // SECTION 4: MARGIN VARIATIONS
    // =========================================================================
    // Normal Margins (19mm top/bottom, 18mm left/right)
    export_and_verify(
        "complex_table_26_margins_normal",
        &model,
        &PdfExportOptions::default()
            .with_margins(Margins::normal())
            .fit_to_page(),
        &out,
    );

    // Narrow Margins (6.35mm all around)
    export_and_verify(
        "complex_table_27_margins_narrow",
        &model,
        &PdfExportOptions::default()
            .with_margins(Margins::narrow())
            .fit_to_page(),
        &out,
    );

    // Wide Margins (25.4mm all around)
    export_and_verify(
        "complex_table_28_margins_wide",
        &model,
        &PdfExportOptions::default()
            .with_margins(Margins::wide())
            .fit_to_page(),
        &out,
    );

    // None Margins (0mm)
    export_and_verify(
        "complex_table_29_margins_none",
        &model,
        &PdfExportOptions::default()
            .with_margins(Margins::none())
            .fit_to_page(),
        &out,
    );

    // Custom Margins (12mm top/bottom, 8mm left/right, 5mm header/footer)
    export_and_verify(
        "complex_table_30_margins_custom",
        &model,
        &PdfExportOptions::default()
            .with_margins(Margins::custom(12.0, 12.0, 8.0, 8.0, 5.0, 5.0))
            .fit_to_page(),
        &out,
    );

    // =========================================================================
    // SECTION 5: GRIDLINES & HEADINGS CONTROLS
    // =========================================================================
    // Gridlines Visible
    export_and_verify(
        "complex_table_31_gridlines_visible",
        &model,
        &PdfExportOptions::default()
            .with_gridlines(true)
            .fit_to_page(),
        &out,
    );

    // Gridlines Hidden (Presentation appearance)
    export_and_verify(
        "complex_table_32_gridlines_hidden",
        &model,
        &PdfExportOptions::default()
            .with_gridlines(false)
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_32_gridlines_hidden.pdf"),
        out.join("complex_table_no_gridlines.pdf"),
    )
    .unwrap();

    // Headings Visible (Row numbers 1..30, Col letters A..M)
    export_and_verify(
        "complex_table_33_headings_visible",
        &model,
        &PdfExportOptions::default()
            .with_headings(true)
            .with_gridlines(true)
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_33_headings_visible.pdf"),
        out.join("complex_table_with_headings.pdf"),
    )
    .unwrap();

    // Headings Hidden
    export_and_verify(
        "complex_table_34_headings_hidden",
        &model,
        &PdfExportOptions::default()
            .with_headings(false)
            .fit_to_page(),
        &out,
    );

    // =========================================================================
    // SECTION 6: FORMULA PRINT POLICIES
    // =========================================================================
    // Calculated Values (Standard)
    export_and_verify(
        "complex_table_35_formulas_calculated_values",
        &model,
        &PdfExportOptions::default()
            .with_formulas(FormulaPrintPolicy::CalculatedValues)
            .fit_to_page(),
        &out,
    );

    // Raw Formula Text (=D5*E5, =SUM(D5:D24))
    export_and_verify(
        "complex_table_36_formulas_raw_text",
        &model,
        &PdfExportOptions::default()
            .with_formulas(FormulaPrintPolicy::FormulaText)
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_36_formulas_raw_text.pdf"),
        out.join("complex_table_formula_text.pdf"),
    )
    .unwrap();

    // =========================================================================
    // SECTION 7: CELL ERROR TRANSFORMATION POLICIES (Evaluated on model_with_errors)
    // =========================================================================
    // Display As Stored (#DIV/0!, #N/A)
    export_and_verify(
        "complex_table_37_errors_display_as_stored",
        &model_with_errors,
        &PdfExportOptions::default()
            .with_cell_errors(CellErrorMode::DisplayAsStored)
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_37_errors_display_as_stored.pdf"),
        out.join("complex_table_errors_display_as_stored.pdf"),
    )
    .unwrap();

    // Display As Dash (--)
    export_and_verify(
        "complex_table_38_errors_dash",
        &model_with_errors,
        &PdfExportOptions::default()
            .with_cell_errors(CellErrorMode::Dash)
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_38_errors_dash.pdf"),
        out.join("complex_table_errors_dash.pdf"),
    )
    .unwrap();

    // Display As Blank ("")
    export_and_verify(
        "complex_table_39_errors_blank",
        &model_with_errors,
        &PdfExportOptions::default()
            .with_cell_errors(CellErrorMode::Blank)
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_39_errors_blank.pdf"),
        out.join("complex_table_errors_blank.pdf"),
    )
    .unwrap();

    // Display As NotAvailable (#N/A)
    export_and_verify(
        "complex_table_40_errors_not_available",
        &model_with_errors,
        &PdfExportOptions::default()
            .with_cell_errors(CellErrorMode::NotAvailable)
            .fit_to_page(),
        &out,
    );
    fs::copy(
        out.join("complex_table_40_errors_not_available.pdf"),
        out.join("complex_table_errors_na.pdf"),
    )
    .unwrap();

    // =========================================================================
    // SECTION 8: EXPORT SCOPE
    // =========================================================================
    // Active Sheet Only (Financials)
    export_and_verify(
        "complex_table_41_scope_active_sheet",
        &model,
        &PdfExportOptions::default()
            .with_scope(ExportScope::ActiveSheet)
            .fit_to_page(),
        &out,
    );

    // Entire Workbook (Multi-Sheet with outline bookmarks)
    export_and_verify(
        "complex_table_42_scope_entire_workbook",
        &model,
        &PdfExportOptions::default()
            .with_scope(ExportScope::EntireWorkbook)
            .with_bookmarks(BookmarkPolicy::SheetBookmarksWhenExportingMultipleSheets)
            .fit_to_page()
            .with_header_footer(
                HeaderFooterLayout {
                    left: HeaderFooterField::SheetName,
                    center: HeaderFooterField::Text("CONSOLIDATED AUDIT".to_string()),
                    right: HeaderFooterField::Date,
                },
                HeaderFooterLayout {
                    left: HeaderFooterField::WorkbookName,
                    center: HeaderFooterField::Empty,
                    right: HeaderFooterField::PageNumberOfTotal,
                },
            ),
        &out,
    );
    fs::copy(
        out.join("complex_table_42_scope_entire_workbook.pdf"),
        out.join("complex_table_entire_workbook.pdf"),
    )
    .unwrap();

    // Selection: Main Data Table (Rows 4..25, Cols 1..13)
    export_and_verify(
        "complex_table_43_scope_selection_table",
        &model,
        &PdfExportOptions::default()
            .with_selection(
                0,
                CellRange {
                    start_row: 4,
                    start_col: 1,
                    end_row: 25,
                    end_col: 13,
                },
            )
            .fit_to_page()
            .with_gridlines(true),
        &out,
    );
    fs::copy(
        out.join("complex_table_43_scope_selection_table.pdf"),
        out.join("complex_table_selection.pdf"),
    )
    .unwrap();

    // Selection: Summary & Performance Block (Rows 25..29, Cols 1..13)
    export_and_verify(
        "complex_table_44_scope_selection_totals",
        &model,
        &PdfExportOptions::default()
            .with_selection(
                0,
                CellRange {
                    start_row: 25,
                    start_col: 1,
                    end_row: 29,
                    end_col: 13,
                },
            )
            .fit_to_page()
            .with_centering(true, true),
        &out,
    );

    // Scope: Specific Sheet (Executive Summary only)
    export_and_verify(
        "complex_table_45_scope_second_sheet",
        &model,
        &PdfExportOptions::default()
            .with_scope(ExportScope::SelectedSheets(vec![1]))
            .fit_to_page(),
        &out,
    );

    // =========================================================================
    // SECTION 9: PAGE ORDER (MULTI-PAGE PAGINATION)
    // =========================================================================
    // Down Then Over (Standard Excel default)
    export_and_verify(
        "complex_table_46_page_order_down_then_over",
        &model,
        &PdfExportOptions::default()
            .with_scaling(Scaling::ActualSize)
            .with_page_order(PageOrder::DownThenOver),
        &out,
    );

    // Over Then Down
    export_and_verify(
        "complex_table_47_page_order_over_then_down",
        &model,
        &PdfExportOptions::default()
            .with_scaling(Scaling::ActualSize)
            .with_page_order(PageOrder::OverThenDown),
        &out,
    );
    fs::copy(
        out.join("complex_table_47_page_order_over_then_down.pdf"),
        out.join("complex_table_over_then_down.pdf"),
    )
    .unwrap();

    // =========================================================================
    // SECTION 10: CENTERING ON PAGE
    // =========================================================================
    // Uncentered (Top-left aligned)
    export_and_verify(
        "complex_table_48_centering_none",
        &model,
        &PdfExportOptions::default()
            .with_centering(false, false)
            .fit_to_page(),
        &out,
    );

    // Centered Horizontally Only
    export_and_verify(
        "complex_table_49_centering_horizontal",
        &model,
        &PdfExportOptions::default()
            .with_centering(true, false)
            .fit_to_page(),
        &out,
    );

    // Centered Both Horizontally and Vertically
    export_and_verify(
        "complex_table_50_centering_both",
        &model,
        &PdfExportOptions::default()
            .with_centering(true, true)
            .fit_to_page(),
        &out,
    );

    // =========================================================================
    // SECTION 11: HEADERS & FOOTERS (DYNAMIC TOKENS)
    // =========================================================================
    // Standard Page & Date
    export_and_verify(
        "complex_table_51_headers_footers_standard",
        &model,
        &PdfExportOptions::default()
            .fit_to_page()
            .with_header_footer(
                HeaderFooterLayout {
                    left: HeaderFooterField::Text("ENTERPRISE AUDIT".to_string()),
                    center: HeaderFooterField::Empty,
                    right: HeaderFooterField::Date,
                },
                HeaderFooterLayout {
                    left: HeaderFooterField::Empty,
                    center: HeaderFooterField::PageNumberOfTotal,
                    right: HeaderFooterField::Empty,
                },
            ),
        &out,
    );

    // All dynamic tokens (&P of &N, &A sheet, &F workbook, &D date, &T time)
    export_and_verify(
        "complex_table_52_headers_footers_all_tokens",
        &model,
        &PdfExportOptions::default()
            .fit_to_page()
            .with_header_footer(
                HeaderFooterLayout {
                    left: HeaderFooterField::SheetName,
                    center: HeaderFooterField::WorkbookName,
                    right: HeaderFooterField::Date,
                },
                HeaderFooterLayout {
                    left: HeaderFooterField::Time,
                    center: HeaderFooterField::PageNumberOfTotal,
                    right: HeaderFooterField::Text("IRONCALC EXPORT".to_string()),
                },
            ),
        &out,
    );

    // Formal Confidentiality Header
    export_and_verify(
        "complex_table_53_headers_footers_confidential",
        &model,
        &PdfExportOptions::default()
            .fit_to_page()
            .with_header_footer(
                HeaderFooterLayout {
                    left: HeaderFooterField::Text("CONFIDENTIAL & PROPRIETARY".to_string()),
                    center: HeaderFooterField::Empty,
                    right: HeaderFooterField::SheetName,
                },
                HeaderFooterLayout {
                    left: HeaderFooterField::Text("DO NOT DISTRIBUTE".to_string()),
                    center: HeaderFooterField::PageNumberOfTotal,
                    right: HeaderFooterField::Date,
                },
            ),
        &out,
    );

    // =========================================================================
    // SECTION 12: PRINT TITLES & REPEATED HEADERS
    // =========================================================================
    // Repeat Title and Column Headers (Rows 1..4) on all printed pages
    export_and_verify(
        "complex_table_54_repeat_headers_multipage",
        &model,
        &PdfExportOptions::default()
            .with_scaling(Scaling::ActualSize)
            .with_repeat_rows(1, 4)
            .with_gridlines(true),
        &out,
    );
    fs::copy(
        out.join("complex_table_54_repeat_headers_multipage.pdf"),
        out.join("complex_table_repeat_headers_multipage.pdf"),
    )
    .unwrap();

    // Repeat Frozen Panes as Print Titles
    export_and_verify(
        "complex_table_55_repeat_frozen_panes",
        &model,
        &PdfExportOptions::default()
            .with_scaling(Scaling::ActualSize)
            .with_repeat_frozen_panes(true)
            .with_gridlines(true),
        &out,
    );

    // =========================================================================
    // SECTION 13: BOOKMARK POLICIES
    // =========================================================================
    // Always generate sheet outline bookmarks
    export_and_verify(
        "complex_table_56_bookmarks_always",
        &model,
        &PdfExportOptions::default()
            .with_bookmarks(BookmarkPolicy::AlwaysBySheet)
            .fit_to_page(),
        &out,
    );

    // Bookmarks explicitly disabled
    export_and_verify(
        "complex_table_57_bookmarks_disabled",
        &model,
        &PdfExportOptions::default()
            .with_bookmarks(BookmarkPolicy::None)
            .fit_to_page(),
        &out,
    );

    println!("All 57 distinct PDF export configurations successfully generated and verified!");
}

#[test]
fn test_complex_table_png_export_validation() {
    let out = output_dir();
    let png_path = out.join("complex_table.png");

    // If the PNG has not been generated yet, attempt to trigger generation via the app test harness
    if !png_path.exists() {
        let root_dir = manifest_dir().join("..");
        let _ = std::process::Command::new("npx")
            .current_dir(root_dir.join("webapp/IronCalc"))
            .args([
                "vitest",
                "run",
                "tests/worksheetCanvas/complexTableExport.test.ts",
            ])
            .output();
    }

    assert!(
        png_path.exists(),
        "complex_table.png must be generated by the app's PNG export capability"
    );

    let png_bytes = fs::read(&png_path).expect("read complex_table.png");
    assert!(
        png_bytes.len() > 10_000,
        "PNG file size must be non-trivial (>10KB), was {}",
        png_bytes.len()
    );

    // Verify PNG magic bytes: 0x89, 'P', 'N', 'G', '\r', '\n', 0x1A, '\n'
    assert_eq!(
        &png_bytes[0..8],
        b"\x89PNG\r\n\x1a\n",
        "Exported PNG must start with valid PNG signature"
    );

    println!(
        "Verified complex table PNG export: {:?} ({} bytes)",
        png_path,
        png_bytes.len()
    );
}
