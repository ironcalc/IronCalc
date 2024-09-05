use std::vec::Vec;

use crate::types::*;

impl Workbook {
    pub fn get_worksheet_names(&self) -> Vec<String> {
        self.worksheets
            .iter()
            .map(|worksheet| worksheet.get_name())
            .collect()
    }
    pub fn get_worksheet_ids(&self) -> Vec<u32> {
        self.worksheets
            .iter()
            .map(|worksheet| worksheet.get_sheet_id())
            .collect()
    }

    pub fn worksheet(&self, worksheet_index: u32) -> Result<&Worksheet, String> {
        self.worksheets
            .get(worksheet_index as usize)
            .ok_or_else(|| "Invalid sheet index".to_string())
    }

    pub fn worksheet_mut(&mut self, worksheet_index: u32) -> Result<&mut Worksheet, String> {
        self.worksheets
            .get_mut(worksheet_index as usize)
            .ok_or_else(|| "Invalid sheet index".to_string())
    }
}
