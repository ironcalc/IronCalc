# Code Context Report for: `TODAY`

This report contains all relevant code snippets for the `TODAY` function, gathered to assist in writing its technical documentation.

## 1. Primary Definition

**File:** `base/src/functions/date_and_time.rs`
**Lines:** `953-969`

```rust
pub(crate) fn fn_today(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
    if !args.is_empty() {
        return CalcResult::Error {
            error: Error::ERROR,
            origin: cell,
            message: "Wrong number of arguments".to_string(),
        };
    }
    match self.current_excel_serial() {
        Some(serial) => CalcResult::Number(serial.floor()),
        None => CalcResult::Error {
            error: Error::ERROR,
            origin: cell,
            message: "Invalid date".to_string(),
        },
    }
}
```

### Helper Method: `current_excel_serial`

**File:** `base/src/functions/date_and_time.rs`
**Lines:** `755-763`

```rust
// Returns the current date/time as an Excel serial number in the model's configured timezone.
// Used by TODAY() and NOW().
fn current_excel_serial(&self) -> Option<f64> {
    let seconds = get_milliseconds_since_epoch() / 1000;
    DateTime::from_timestamp(seconds, 0).map(|dt| {
        let local_time = dt.with_timezone(&self.tz);
        let days_from_1900 = local_time.num_days_from_ce() - EXCEL_DATE_BASE;
        let fraction = (local_time.num_seconds_from_midnight() as f64) / (60.0 * 60.0 * 24.0);
        days_from_1900 as f64 + fraction
    })
}
```

## 2. Unit Tests

### Test Case 1: Basic functionality

**File:** `base/src/test/test_today.rs`
**Lines:** `10-19`

```rust
#[test]
fn today_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=TODAY()");
    model._set("A2", "=TEXT(A1, \"yyyy/m/d\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"08/11/2022");
    assert_eq!(model._get_text("A2"), *"2022/11/8");
}
```

### Test Case 2: Timezone handling error

**File:** `base/src/test/test_today.rs`
**Lines:** `21-25`

```rust
#[test]
fn today_with_wrong_tz() {
    let model = Model::new_empty("model", "en", "Wrong Timezone");
    assert!(model.is_err());
}
```

### Test Case 3: TODAY vs NOW comparison (UTC timezone)

**File:** `base/src/test/test_today.rs`
**Lines:** `27-37`

```rust
#[test]
fn now_basic_utc() {
    mock_time::set_mock_time(TIMESTAMP_2023);
    let mut model = Model::new_empty("model", "en", "UTC").unwrap();
    model._set("A1", "=TODAY()");
    model._set("A2", "=NOW()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"20/03/2023");
    assert_eq!(model._get_text("A2"), *"45005.572511574");
}
```

### Test Case 4: TODAY with timezone offset (Europe/Berlin)

**File:** `base/src/test/test_today.rs`
**Lines:** `39-50`

```rust
#[test]
fn now_basic_europe_berlin() {
    mock_time::set_mock_time(TIMESTAMP_2023);
    let mut model = Model::new_empty("model", "en", "Europe/Berlin").unwrap();
    model._set("A1", "=TODAY()");
    model._set("A2", "=NOW()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"20/03/2023");
    // This is UTC + 1 hour: 45005.572511574 + 1/24
    assert_eq!(model._get_text("A2"), *"45005.614178241");
}
```

### Test Constants

**File:** `base/src/test/test_today.rs`
**Lines:** `7-8`

```rust
// 14:44 20 Mar 2023 Berlin
const TIMESTAMP_2023: i64 = 1679319865208;
```

## 3. Usage Examples

### Example 1: Formula pasting - TODAY with YEAR function

**File:** `base/src/test/user_model/test_paste_csv.rs`
**Lines:** `30-49`

```rust
#[test]
fn csv_paste_formula() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();

    let csv = "=YEAR(TODAY())";
    let area = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    model.set_selected_cell(1, 1).unwrap();
    model.paste_csv_string(&area, csv).unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("2022".to_string())
    );
    assert_eq!([1, 1, 1, 1], model.get_selected_view().range);
}
```

## 4. Related Type Definitions

### Type: `CellReferenceIndex`

**File:** `base/src/expressions/types.rs`
**Lines:** `37-42`

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct CellReferenceIndex {
    pub sheet: u32,
    pub column: i32,
    pub row: i32,
}
```

### Type: `Node`

**File:** `base/src/expressions/parser/mod.rs`
**Lines:** `106-199`

```rust
#[derive(PartialEq, Clone, Debug)]
pub enum Node {
    BooleanKind(bool),
    NumberKind(f64),
    StringKind(String),
    ReferenceKind {
        sheet_name: Option<String>,
        sheet_index: u32,
        absolute_row: bool,
        absolute_column: bool,
        row: i32,
        column: i32,
    },
    RangeKind {
        sheet_name: Option<String>,
        sheet_index: u32,
        absolute_row1: bool,
        absolute_column1: bool,
        row1: i32,
        column1: i32,
        absolute_row2: bool,
        absolute_column2: bool,
        row2: i32,
        column2: i32,
    },
    WrongReferenceKind {
        sheet_name: Option<String>,
        absolute_row: bool,
        absolute_column: bool,
        row: i32,
        column: i32,
    },
    WrongRangeKind {
        sheet_name: Option<String>,
        absolute_row1: bool,
        absolute_column1: bool,
        row1: i32,
        column1: i32,
        absolute_row2: bool,
        absolute_column2: bool,
        row2: i32,
        column2: i32,
    },
    OpRangeKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    OpConcatenateKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    OpSumKind {
        kind: token::OpSum,
        left: Box<Node>,
        right: Box<Node>,
    },
    OpProductKind {
        kind: token::OpProduct,
        left: Box<Node>,
        right: Box<Node>,
    },
    OpPowerKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    FunctionKind {
        kind: Function,
        args: Vec<Node>,
    },
    InvalidFunctionKind {
        name: String,
        args: Vec<Node>,
    },
    ArrayKind(Vec<Vec<ArrayNode>>),
    DefinedNameKind(DefinedNameS),
    TableNameKind(String),
    WrongVariableKind(String),
    ImplicitIntersection {
        automatic: bool,
        child: Box<Node>,
    },
    CompareKind {
        kind: OpCompare,
        left: Box<Node>,
        right: Box<Node>,
    },
    UnaryKind {
        kind: OpUnary,
        right: Box<Node>,
    },
    ErrorKind(token::Error),
    ParseErrorKind {
        formula: String,
        message: String,
        position: usize,
    },
    // ... (truncated for display - see full enum in source file)
}
```

### Type: `CalcResult`

**File:** `base/src/calc_result.rs`
**Lines:** `12-28`

```rust
#[derive(Clone)]
pub(crate) enum CalcResult {
    String(String),
    Number(f64),
    Boolean(bool),
    Error {
        error: Error,
        origin: CellReferenceIndex,
        message: String,
    },
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
    EmptyCell,
    EmptyArg,
    Array(Vec<Vec<ArrayNode>>),
}
```

## 5. Function Registration

### Registration in Function Enum

**File:** `base/src/functions/mod.rs`
**Lines:** `206`

```rust
pub enum Function {
    // ... other functions
    Today,
    // ... other functions
}
```

### String to Function Mapping

**File:** `base/src/functions/mod.rs`
**Lines:** `805`

```rust
pub fn get_function(name: &str) -> Option<Function> {
    match name.to_ascii_uppercase().as_str() {
        // ... other cases
        "TODAY" => Some(Function::Today),
        // ... other cases
    }
}
```

### Function to String Mapping

**File:** `base/src/functions/mod.rs`
**Lines:** `1045`

```rust
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... other cases
            Function::Today => write!(f, "TODAY"),
            // ... other cases
        }
    }
}
```

### Function Evaluation Dispatch

**File:** `base/src/functions/mod.rs`
**Lines:** `1322`

```rust
pub(crate) fn evaluate_function(
    &mut self,
    kind: &Function,
    args: &[Node],
    cell: CellReferenceIndex,
) -> CalcResult {
    match kind {
        // ... other cases
        Function::Today => self.fn_today(args, cell),
        // ... other cases
    }
}
```

### Function Iterator Registration

**File:** `base/src/functions/mod.rs`
**Lines:** `467` (within `Function::into_iter()`)

```rust
pub fn into_iter() -> IntoIter<Function, 256> {
    [
        // ... other functions
        Function::Today,
        // ... other functions
    ]
    .into_iter()
}
```

## 6. Key Implementation Details

### Constants Used

**File:** `base/src/functions/date_and_time.rs`

- `EXCEL_DATE_BASE`: Used to convert from Common Era days to Excel serial numbers (referenced at line 759)

### Dependencies

The `TODAY` function depends on:

1. **`get_milliseconds_since_epoch()`**: Retrieves the current timestamp
2. **`chrono::DateTime`**: For date/time conversion
3. **Model's timezone (`self.tz`)**: Configured timezone for the spreadsheet model
4. **Excel date system**: Uses serial number system where January 1, 1900 is day 1

### Behavior Notes

1. **No Arguments**: The function takes no arguments. If any arguments are provided, it returns an error.
2. **Returns Floor Value**: Unlike `NOW()` which returns the full serial number with time fraction, `TODAY()` returns only the integer part (date without time) via `.floor()`.
3. **Timezone Aware**: The function respects the model's configured timezone setting.
4. **Excel Compatibility**: Returns Excel serial date numbers for compatibility with Excel files.
5. **Error Handling**: Returns `Error::ERROR` for invalid timezones or if timestamp conversion fails.

## 7. Related Functions

- **`NOW()`** (base/src/functions/date_and_time.rs:971-987): Similar to TODAY but includes the time fraction
- **`DATE()`** (base/src/functions/date_and_time.rs:564-605): Constructs a date from year, month, and day components
