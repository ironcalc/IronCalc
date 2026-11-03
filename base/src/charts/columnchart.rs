use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::charts::common::{
    Bars, CategoryAxis, DataLabels, DataSource, Legend, Stacking, Title, ValueAxis,
};
use crate::types::Color;

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct ColumnChart {
    pub title: Option<Title>,
    pub legend: Legend,
    pub stacking: Stacking,
    /// The spacing of the columns; stacked columns only use the gap.
    pub bars: Bars,
    pub category_axis: CategoryAxis,
    pub value_axis: ValueAxis,
    pub data: DataSource,
    /// One color per series, by position. Series past the end of the list
    /// take a color from the default palette.
    pub colors: Vec<Color>,
    /// The labels of each series, by position. `None`, or a series past the
    /// end of the list, has no labels.
    pub data_labels: Vec<Option<DataLabels>>,
}
