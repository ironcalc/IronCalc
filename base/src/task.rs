use serde::{Deserialize, Serialize};

use crate::{expressions::types::CellReferenceIndex, finance::provider::Ticker};

/// A side-effect that needs to be executed outside the pure evaluation engine.
///
/// When a formula cannot complete because it needs external data (e.g. a
/// network fetch), it pushes a `Task` onto `Model::pending_tasks` and returns
/// a `#N/A` placeholder. The caller drains the tasks after `evaluate()`,
/// executes them asynchronously, feeds the results back via
/// `Model::complete_task()`, and re-evaluates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Task {
    /// Fetch financial data (stock price, forex rate, crypto quote).
    FinanceFetch(FinanceFetchTask),
}

/// A task for fetching financial data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinanceFetchTask {
    /// The ticker to query (e.g. `Ticker::Stock("AAPL")`).
    pub ticker: Ticker,
    /// The attribute to fetch: `"price"`, `"open"`, `"high"`, `"low"`,
    /// `"close"`, `"volume"`.
    pub attribute: String,
    /// The cell that triggered the fetch, so the caller can
    /// invalidate / re-evaluate it after the data arrives.
    pub cell: CellReferenceIndex,
}
