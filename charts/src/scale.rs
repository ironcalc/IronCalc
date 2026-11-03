//! The value axis: which numbers it runs between and where the ticks are.
//!
//! Follows Excel's automatic scaling as far as it is known: the axis starts at
//! zero unless the data is squeezed into the top sixth of its range, the data
//! gets five percent of headroom, and both ends land on a "nice" step of 1, 2
//! or 5 times a power of ten. How many ticks fit is for the layout to say,
//! since it depends on the height of the plot and the size of the labels.

use ironcalc_base::charts::{ChartData, ColumnChart, Stacking};
use ironcalc_base::formatter::format::format_number;
use ironcalc_base::locale::Locale;

/// A value axis from `min` to `max` with a tick every `step`. Both ends are
/// multiples of `step`.
#[derive(Debug, PartialEq, Clone)]
pub struct Scale {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

/// Rounds away the noise of adding steps up: 0.1 + 0.2 is 0.3 again.
fn clean(value: f64) -> f64 {
    format!("{value:.12e}").parse().unwrap_or(value)
}

/// The smallest step of 1, 2 or 5 times a power of ten that covers `range`
/// in at most `max_ticks` intervals.
fn nice_step(range: f64, max_ticks: usize) -> f64 {
    let max_ticks = max_ticks.max(1) as f64;
    let rough = range / max_ticks;
    let magnitude = 10f64.powf(rough.log10().floor());
    for factor in [1.0, 2.0, 5.0, 10.0] {
        let step = clean(factor * magnitude);
        if range / step <= max_ticks {
            return step;
        }
    }
    clean(10.0 * magnitude)
}

impl Scale {
    /// The axis Excel would pick for data between `data_min` and `data_max`,
    /// with at most `max_ticks` intervals between major ticks.
    pub fn automatic(data_min: f64, data_max: f64, max_ticks: usize) -> Scale {
        let (mut low, mut high) = (data_min.min(data_max), data_min.max(data_max));
        if low == high {
            if high > 0.0 {
                low = 0.0;
            } else if high < 0.0 {
                high = 0.0;
            } else {
                // Nothing to show: Excel draws an empty axis from 0 to 1
                return Scale::covering(0.0, 1.0, max_ticks);
            }
        }
        // The axis includes zero unless the data lives in the top sixth
        let range = high - low;
        if low >= 0.0 && range > high / 6.0 {
            low = 0.0;
        }
        if high <= 0.0 && range > -low / 6.0 {
            high = 0.0;
        }
        // Five percent of headroom, but never past zero
        let padding = 0.05 * (high - low);
        let low = if low == 0.0 { 0.0 } else { low - padding };
        let high = if high == 0.0 { 0.0 } else { high + padding };
        Scale::covering(low, high, max_ticks)
    }

    /// The axis for a fixed interval, such as 0 to 1 for a percent stacked
    /// chart: no headroom, the ends only extended to land on a step.
    pub fn covering(low: f64, high: f64, max_ticks: usize) -> Scale {
        let step = nice_step(high - low, max_ticks);
        Scale {
            min: clean((low / step).floor() * step),
            max: clean((high / step).ceil() * step),
            step,
        }
    }

    /// The major ticks, from `min` to `max` included.
    pub fn ticks(&self) -> Vec<f64> {
        let count = ((self.max - self.min) / self.step).round() as usize;
        (0..=count)
            .map(|index| clean(self.min + index as f64 * self.step))
            .collect()
    }

    /// Where `value` sits along the axis: 0 at `min`, 1 at `max`.
    pub fn fraction(&self, value: f64) -> f64 {
        (value - self.min) / (self.max - self.min)
    }

    /// Where zero sits along the axis, clamped to the ends: the bars grow
    /// from here.
    pub fn baseline_fraction(&self) -> f64 {
        self.fraction(0.0).clamp(0.0, 1.0)
    }
}

/// The lowest and highest value the axis has to show, depending on how the
/// series stack. `(0, 0)` when there is no data at all.
pub fn value_bounds(data: &ChartData, stacking: &Stacking) -> (f64, f64) {
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    let mut extend = |value: f64| {
        low = low.min(value);
        high = high.max(value);
    };
    match stacking {
        Stacking::None => {
            for value in data.series.iter().flat_map(|s| s.values.iter().flatten()) {
                extend(*value);
            }
        }
        Stacking::Standard => {
            // Positive and negative values stack on their own side of zero
            let points = data.series.iter().map(|s| s.values.len()).max();
            for index in 0..points.unwrap_or(0) {
                let (mut positive, mut negative) = (0.0, 0.0);
                for value in data
                    .series
                    .iter()
                    .filter_map(|s| s.values.get(index)?.as_ref())
                {
                    if *value >= 0.0 {
                        positive += value;
                    } else {
                        negative += value;
                    }
                }
                extend(positive);
                extend(negative);
            }
        }
        Stacking::Full => {
            let values = || data.series.iter().flat_map(|s| s.values.iter().flatten());
            if values().any(|v| *v > 0.0) {
                extend(1.0);
            }
            if values().any(|v| *v < 0.0) {
                extend(-1.0);
            }
            if values().next().is_none() {
                extend(1.0);
            }
            extend(0.0);
        }
    }
    if low > high {
        (0.0, 0.0)
    } else {
        (low, high)
    }
}

/// The scale of the value axis of `chart`.
pub fn value_scale(chart: &ColumnChart, data: &ChartData, max_ticks: usize) -> Scale {
    let (low, high) = value_bounds(data, &chart.stacking);
    match chart.stacking {
        Stacking::Full => Scale::covering(low, high, max_ticks),
        _ => Scale::automatic(low, high, max_ticks),
    }
}

/// The number format of the tick labels: the axis' own, or the one of the
/// source when linked, or percentages for a percent stacked chart.
pub fn value_axis_format(chart: &ColumnChart, data: &ChartData) -> String {
    if let Some(format) = &chart.value_axis.labels.number_format {
        return format.clone();
    }
    if chart.stacking == Stacking::Full {
        return "0%".to_string();
    }
    data.series
        .first()
        .map(|s| s.number_format.clone())
        .unwrap_or_else(|| "General".to_string())
}

/// The label of every tick of `scale`.
pub fn tick_labels(scale: &Scale, number_format: &str, locale: &Locale) -> Vec<String> {
    scale
        .ticks()
        .iter()
        .map(|tick| format_number(*tick, number_format, locale).text)
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ironcalc_base::charts::SeriesData;
    use ironcalc_base::locale::get_locale;

    fn scale(min: f64, max: f64, step: f64) -> Scale {
        Scale { min, max, step }
    }

    #[test]
    fn positive_data_starts_at_zero_with_headroom() {
        // 234 + 5% = 245.7, the next step of 50 is 250
        assert_eq!(Scale::automatic(23.0, 234.0, 8), scale(0.0, 250.0, 50.0));
        // 456 + 5% = 478.8: steps of 50 would be 10 ticks, so steps of 100
        assert_eq!(Scale::automatic(23.0, 456.0, 8), scale(0.0, 500.0, 100.0));
        assert_eq!(Scale::automatic(23.0, 456.0, 10), scale(0.0, 500.0, 50.0));
        // A maximum right on a tick still gets headroom
        assert_eq!(Scale::automatic(0.0, 200.0, 8), scale(0.0, 250.0, 50.0));
    }

    #[test]
    fn data_squeezed_at_the_top_does_not_start_at_zero() {
        assert_eq!(
            Scale::automatic(950.0, 1000.0, 8),
            scale(940.0, 1010.0, 10.0)
        );
    }

    #[test]
    fn negative_and_mixed_data() {
        assert_eq!(Scale::automatic(-234.0, -23.0, 8), scale(-250.0, 0.0, 50.0));
        assert_eq!(Scale::automatic(-3.0, 97.0, 8), scale(-20.0, 120.0, 20.0));
    }

    #[test]
    fn degenerate_data() {
        assert_eq!(Scale::automatic(0.0, 0.0, 8), scale(0.0, 1.0, 0.2));
        assert_eq!(Scale::automatic(5.0, 5.0, 8), scale(0.0, 6.0, 1.0));
        assert_eq!(Scale::automatic(-5.0, -5.0, 8), scale(-6.0, 0.0, 1.0));
    }

    #[test]
    fn ticks_are_clean_numbers() {
        assert_eq!(
            Scale::covering(0.0, 1.0, 8).ticks(),
            vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
        );
        assert_eq!(
            Scale::covering(0.0, 1.0, 10).ticks(),
            vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
        );
        let scale = scale(-20.0, 120.0, 20.0);
        assert_eq!(scale.ticks().len(), 8);
        assert_eq!(scale.fraction(120.0), 1.0);
        assert!((scale.baseline_fraction() - 20.0 / 140.0).abs() < 1e-12);
        assert_eq!(Scale::automatic(0.0, 200.0, 8).baseline_fraction(), 0.0);
    }

    fn data(series: &[&[Option<f64>]]) -> ChartData {
        ChartData {
            categories: vec![],
            series: series
                .iter()
                .map(|values| SeriesData {
                    name: "".to_string(),
                    values: values.to_vec(),
                    number_format: "#,##0".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn bounds_depend_on_stacking() {
        let data = data(&[
            &[Some(10.0), Some(-5.0), None],
            &[Some(20.0), Some(-15.0), Some(3.0)],
        ]);
        assert_eq!(value_bounds(&data, &Stacking::None), (-15.0, 20.0));
        // Stacked: 10 + 20 on top, -5 - 15 below
        assert_eq!(value_bounds(&data, &Stacking::Standard), (-20.0, 30.0));
        assert_eq!(value_bounds(&data, &Stacking::Full), (-1.0, 1.0));
        let positive = data_positive();
        assert_eq!(value_bounds(&positive, &Stacking::Full), (0.0, 1.0));
        assert_eq!(
            value_bounds(&self::data(&[&[None]]), &Stacking::None),
            (0.0, 0.0)
        );
        assert_eq!(value_bounds(&self::data(&[]), &Stacking::Full), (0.0, 1.0));
    }

    fn data_positive() -> ChartData {
        data(&[&[Some(10.0), Some(5.0)], &[Some(1.0), Some(2.0)]])
    }

    #[test]
    fn labels_follow_the_number_format() {
        let locale = get_locale("en").unwrap();
        assert_eq!(
            tick_labels(&scale(0.0, 2000.0, 500.0), "#,##0", locale),
            vec!["0", "500", "1,000", "1,500", "2,000"]
        );
        assert_eq!(
            tick_labels(&Scale::covering(0.0, 1.0, 8), "0%", locale),
            vec!["0%", "20%", "40%", "60%", "80%", "100%"]
        );
        assert_eq!(
            tick_labels(&scale(0.0, 0.3, 0.1), "general", locale),
            vec!["0", "0.1", "0.2", "0.3"]
        );
    }

    #[test]
    fn the_axis_format_falls_back_to_the_source() {
        let mut chart = crate::test_utils::column_chart();
        let data = data_positive();
        assert_eq!(value_axis_format(&chart, &data), "#,##0");
        chart.stacking = Stacking::Full;
        assert_eq!(value_axis_format(&chart, &data), "0%");
        chart.value_axis.labels.number_format = Some("0.00".to_string());
        assert_eq!(value_axis_format(&chart, &data), "0.00");
    }
}
