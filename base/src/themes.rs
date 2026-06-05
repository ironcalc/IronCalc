use crate::types::Theme;

//   ┌──────────────┬───────────────────────────────────────────────────────┐
//   │    Theme     │                       Character                       │
//   ├──────────────┼───────────────────────────────────────────────────────┤
//   │ Office       │ The Excel default — slate navy, warm orange, sky blue │
//   ├──────────────┼───────────────────────────────────────────────────────┤
//   │ Retrospect   │ Muted red, earthy green, soft purple                  │
//   ├──────────────┼───────────────────────────────────────────────────────┤
//   │ Facet        │ Vivid lime greens, gold, and orange-red pops          │
//   ├──────────────┼───────────────────────────────────────────────────────┤
//   │ Ion          │ Cool teals, ocean blues, and deep greens              │
//   ├──────────────┼───────────────────────────────────────────────────────┤
//   │ Metropolitan │ Urban indigo, cerulean, and slate                     │
//   └──────────────┴───────────────────────────────────────────────────────┘

macro_rules! theme {
    (
        name: $name:literal,
        dk1: $dk1:literal, lt1: $lt1:literal,
        dk2: $dk2:literal, lt2: $lt2:literal,
        accent1: $a1:literal, accent2: $a2:literal,
        accent3: $a3:literal, accent4: $a4:literal,
        accent5: $a5:literal, accent6: $a6:literal,
        hlink: $hl:literal, fol_hlink: $fhl:literal $(,)?
    ) => {
        Theme {
            name: $name.to_string(),
            dk1: $dk1.to_string(),
            lt1: $lt1.to_string(),
            dk2: $dk2.to_string(),
            lt2: $lt2.to_string(),
            accent1: $a1.to_string(),
            accent2: $a2.to_string(),
            accent3: $a3.to_string(),
            accent4: $a4.to_string(),
            accent5: $a5.to_string(),
            accent6: $a6.to_string(),
            hlink: $hl.to_string(),
            fol_hlink: $fhl.to_string(),
        }
    };
}

pub fn builtin_themes() -> Vec<Theme> {
    vec![
        // Microsoft Office default
        theme! {
            name: "Office",
            dk1: "#000000", lt1: "#FFFFFF",
            dk2: "#44546A", lt2: "#E7E6E6",
            accent1: "#4472C4", accent2: "#ED7D31",
            accent3: "#A5A5A5", accent4: "#FFC000",
            accent5: "#5B9BD5", accent6: "#70AD47",
            hlink: "#0563C1", fol_hlink: "#954F72",
        },
        // Warm reds and earthy greens
        theme! {
            name: "Retrospect",
            dk1: "#000000", lt1: "#FFFFFF",
            dk2: "#3F3F3F", lt2: "#EEECE1",
            accent1: "#C0504D", accent2: "#9BBB59",
            accent3: "#8064A2", accent4: "#4BACC6",
            accent5: "#F79646", accent6: "#7F7F7F",
            hlink: "#C0504D", fol_hlink: "#8064A2",
        },
        // Vivid greens and orange-red pops
        theme! {
            name: "Facet",
            dk1: "#000000", lt1: "#FFFFFF",
            dk2: "#404040", lt2: "#F2F2F2",
            accent1: "#90C226", accent2: "#54A021",
            accent3: "#E6B91E", accent4: "#E76618",
            accent5: "#C42F2F", accent6: "#917EAC",
            hlink: "#0563C1", fol_hlink: "#954F72",
        },
        // Cool teals and blues
        theme! {
            name: "Ion",
            dk1: "#000000", lt1: "#FFFFFF",
            dk2: "#0E5484", lt2: "#E8F5FB",
            accent1: "#1CADE4", accent2: "#2683C6",
            accent3: "#27CED7", accent4: "#42BA97",
            accent5: "#3E8853", accent6: "#62A39F",
            hlink: "#0563C1", fol_hlink: "#954F72",
        },
        // Urban blues and indigos
        theme! {
            name: "Metropolitan",
            dk1: "#000000", lt1: "#FFFFFF",
            dk2: "#323E4F", lt2: "#E8EBF0",
            accent1: "#4763AB", accent2: "#3A96CD",
            accent3: "#66C9D5", accent4: "#00AEAD",
            accent5: "#499478", accent6: "#5E6D73",
            hlink: "#4763AB", fol_hlink: "#499478",
        },
    ]
}
