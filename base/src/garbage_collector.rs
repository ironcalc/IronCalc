use crate::model::Model;

// The gc (Garbage Collector) cleans up leftover elements in the workbook:
// * Strings that are no longe reachable
// * Styles that are no longer reachable
// * ...

impl Model {
    pub(crate) fn gc(&mut self) -> Result<(), String> {
        todo!()
    }
}