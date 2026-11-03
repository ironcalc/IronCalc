use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::charts::columnchart::ColumnChart;

/// Every kind of chart IronCalc knows how to draw. Each variant carries the
/// full definition of a chart of that kind.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Chart {
    ColumnChart(ColumnChart),
    // LineChart(LineChart),
}
