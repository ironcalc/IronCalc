use ironcalc_base::types::Theme;

pub(crate) fn get_theme_xml(theme: &Theme) -> String {
    format!(
        include_str!("theme1.xml"),
        name = &theme.name,
        dk1 = &theme.dk1.trim_start_matches('#'),
        lt1 = &theme.lt1.trim_start_matches('#'),
        dk2 = &theme.dk2.trim_start_matches('#'),
        lt2 = &theme.lt2.trim_start_matches('#'),
        accent1 = &theme.accent1.trim_start_matches('#'),
        accent2 = &theme.accent2.trim_start_matches('#'),
        accent3 = &theme.accent3.trim_start_matches('#'),
        accent4 = &theme.accent4.trim_start_matches('#'),
        accent5 = &theme.accent5.trim_start_matches('#'),
        accent6 = &theme.accent6.trim_start_matches('#'),
        hlink = &theme.hlink.trim_start_matches('#'),
        folHlink = &theme.fol_hlink.trim_start_matches('#')
    )
}
