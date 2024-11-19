use crate::functions::Function;

use super::{CellReferenceIndex, Node};

/*

# NOTES on the Implicit Intersection operator: @

 Sometimes we obtain a range where we expected a single argument. This can happen:

 * As an argument of a function, eg: `SIN(A1:A5)`
 * As the result of a computation of a formula `=A1:A5`

 In previous versions of the Friendly Giant the spreadsheet engine would perform an operation called _implicit intersection_
 that tries to find a single cell within the range. It works by picking a cell in the range that is the same row or the same column
 as the cell. If there is just one we return that otherwise we return the `#REF!` error.

 Examples:

 * Siting on `C3` the formula `=D1:D5` will return `D3`
 * Sitting on `C3` the formula `=D:D` will return `D3`
 * Sitting on `C3` the formula `=A1:A7` will return `A3`
 * Sitting on `C3` the formula `=A5:A8` will return `#REF!`
 * Sitting on `C3` the formula `D1:G7` will return `#REF!`

 Today's version of the engine will result in a dynamic array spilling the result through several cells.
 To force the old behaviour we can use the _implicit intersection operator_: @

 * `=@A1:A7` or `=SIN(@A1:A7)

 When parsing formulas that come form old workbooks this is done automatically.
 We call this version of the II operator the _automatic_ II operator.

 We can also insert the II operator in places where before was impossible:

 * `=SUM(@A1:A7)`

 This formulas will not be compatible with old versions of the engine. The FG will stringify this as `=SUM(_xlfn.SIMPLE(A1:A7))`.
 */

/// Transverses the formula tree adding the implicit intersection operator in all arguments of functions that
/// expect a scalar but get a range.
///  * A:A => @A:A
///  * SIN(A1:D1) => SIN(@A1:D1)
///
/// Assumes formula return a scalar
pub fn add_implicit_intersection(node: &mut Node, cell: &CellReferenceIndex, add: bool) {
    match node {
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::EmptyArgKind
        | Node::ParseErrorKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::InvalidFunctionKind { .. }
        | Node::ArrayKind(_)
        | Node::ReferenceKind { .. } => {}
        Node::ImplicitIntersection { child, .. } => {
            // We need to check wether the II can be automatic or not
            let mut new_node = child.as_ref().clone();
            add_implicit_intersection(&mut new_node, cell, add);
            if matches!(&new_node, Node::ImplicitIntersection { .. }) {
                *node = new_node
            }
        }
        Node::RangeKind {
            row1,
            column1,
            row2,
            column2,
            sheet_name,
            sheet_index,
            absolute_row1,
            absolute_column1,
            absolute_row2,
            absolute_column2,
        } => {
            if add {
                *node = Node::ImplicitIntersection {
                    automatic: true,
                    child: Box::new(Node::RangeKind {
                        sheet_name: sheet_name.clone(),
                        sheet_index: *sheet_index,
                        absolute_row1: *absolute_row1,
                        absolute_column1: *absolute_column1,
                        row1: *row1,
                        column1: *column1,
                        absolute_row2: *absolute_row2,
                        absolute_column2: *absolute_column2,
                        row2: *row2,
                        column2: *column2,
                    }),
                };
            }
        }
        Node::OpRangeKind { left, right } => {
            if add {
                *node = Node::ImplicitIntersection {
                    automatic: true,
                    child: Box::new(Node::OpRangeKind {
                        left: left.clone(),
                        right: right.clone(),
                    }),
                }
            }
        }

        // operations
        Node::UnaryKind { right, .. } => add_implicit_intersection(right, cell, add),
        Node::OpConcatenateKind { left, right }
        | Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right, .. }
        | Node::CompareKind { left, right, .. } => {
            add_implicit_intersection(left, cell, add);
            add_implicit_intersection(right, cell, add);
        }
        // defined names
        Node::VariableKind(_) => {
            // noop for now
        }
        Node::FunctionKind { kind, args } => {
            let arg_count = args.len();
            let signature = get_function_args_signature(kind, arg_count);
            for index in 0..arg_count {
                if matches!(signature[index], Signature::Scalar)
                    && matches!(
                        run_static_analysis_on_node(&args[index], cell),
                        StaticResult::Range(_, _) | StaticResult::Unknown
                    )
                {
                    add_implicit_intersection(&mut args[index], cell, true);
                } else {
                    add_implicit_intersection(&mut args[index], cell, false);
                }
            }
            if add
                && matches!(
                    run_static_analysis_on_node(node, cell),
                    StaticResult::Range(_, _) | StaticResult::Unknown
                )
            {
                *node = Node::ImplicitIntersection {
                    automatic: true,
                    child: Box::new(node.clone()),
                }
            }
        }
    };
}

pub(crate) enum StaticResult {
    Scalar,
    Array(i32, i32),
    Range(i32, i32),
    Unknown,
    // TODO: What if one of the dimensions is known?
    // what if the dimensions are unknown but bounded?
}

fn static_analysis_op_nodes(left: &Node, right: &Node, cell: &CellReferenceIndex) -> StaticResult {
    let lhs = run_static_analysis_on_node(left, cell);
    let rhs = run_static_analysis_on_node(right, cell);
    match (lhs, rhs) {
        (StaticResult::Scalar, StaticResult::Scalar) => StaticResult::Scalar,
        (StaticResult::Scalar, StaticResult::Array(a, b) | StaticResult::Range(a, b)) => {
            StaticResult::Array(a, b)
        }

        (StaticResult::Array(a, b) | StaticResult::Range(a, b), StaticResult::Scalar) => {
            StaticResult::Array(a, b)
        }
        (
            StaticResult::Array(a1, b1) | StaticResult::Range(a1, b1),
            StaticResult::Array(a2, b2) | StaticResult::Range(a2, b2),
        ) => StaticResult::Array(a1.max(a2), b1.max(b2)),

        (_, StaticResult::Unknown) => StaticResult::Unknown,
        (StaticResult::Unknown, _) => StaticResult::Unknown,
    }
}

// Returns:
//  * Scalar if we can proof the result of the evaluation is a scalar
//  * Array(a, b) if we know it will be an a x b array.
//  * Range(a, b) if we know it will be a a x b range.
//  * Unknown if we cannot guaranty either
fn run_static_analysis_on_node(node: &Node, cell: &CellReferenceIndex) -> StaticResult {
    match node {
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::EmptyArgKind => StaticResult::Scalar,
        Node::UnaryKind { right, .. } => run_static_analysis_on_node(right, cell),
        Node::ParseErrorKind { .. } => {
            // StaticResult::Unknown is also valid
            StaticResult::Scalar
        }
        Node::WrongReferenceKind { .. } => {
            // StaticResult::Unknown is also valid
            StaticResult::Scalar
        }
        Node::WrongRangeKind { .. } => {
            // StaticResult::Unknown or Array is also valid
            StaticResult::Scalar
        }
        Node::InvalidFunctionKind { .. } => {
            // StaticResult::Unknown is also valid
            StaticResult::Scalar
        }
        Node::ArrayKind(array) => {
            todo!()
            // let n = array.len() as i32;
            // let m = if n > 0 { array[0].len() as i32 } else { 0 };
            // StaticResult::Array(n, m)
        }
        Node::RangeKind {
            row1,
            column1,
            row2,
            column2,
            ..
        } => StaticResult::Range(row2 - row1, column2 - column1),
        Node::OpRangeKind { .. } => {
            // TODO: We could do a bit better here
            StaticResult::Unknown
        }
        Node::ReferenceKind { .. } => StaticResult::Scalar,

        // binary operations
        Node::OpConcatenateKind { left, right } => static_analysis_op_nodes(left, right, cell),
        Node::OpSumKind { left, right, .. } => static_analysis_op_nodes(left, right, cell),
        Node::OpProductKind { left, right, .. } => static_analysis_op_nodes(left, right, cell),
        Node::OpPowerKind { left, right, .. } => static_analysis_op_nodes(left, right, cell),
        Node::CompareKind { left, right, .. } => static_analysis_op_nodes(left, right, cell),

        // defined names
        Node::VariableKind(_s) => StaticResult::Unknown,
        Node::FunctionKind { kind, args } => static_analysis_on_function(kind, args, cell),
        Node::ImplicitIntersection { .. } => StaticResult::Scalar,
    }
}

// If all the arguments are scalars the function will return a scalar
// If any of the arguments is a range or an array it will return an array
fn scalar_arguments(args: &[Node], cell: &CellReferenceIndex) -> StaticResult {
    let mut n = 0;
    let mut m = 0;
    for arg in args {
        match run_static_analysis_on_node(arg, cell) {
            StaticResult::Scalar => {
                // noop
            }
            StaticResult::Array(a, b) | StaticResult::Range(a, b) => {
                n = n.max(a);
                m = m.max(b);
            }
            StaticResult::Unknown => return StaticResult::Unknown,
        }
    }
    if n == 0 && m == 0 {
        return StaticResult::Scalar;
    }
    StaticResult::Array(n, m)
}

// We only care if the function can return a range or not
fn not_implemented(_args: &[Node], _cell: &CellReferenceIndex) -> StaticResult {
    StaticResult::Scalar
}

fn static_analysis_offset(args: &[Node], _cell: &CellReferenceIndex) -> StaticResult {
    // If first argument is a single cell reference and there are no4th and 5th argument,
    // or they are 1, then it is a scalar
    let arg_count = args.len();
    if arg_count < 3 {
        // Actually an error
        return StaticResult::Scalar;
    }
    if !matches!(args[0], Node::ReferenceKind { .. }) {
        return StaticResult::Unknown;
    }
    if arg_count == 3 {
        return StaticResult::Scalar;
    }
    match args[3] {
        Node::NumberKind(f) => {
            if f != 1.0 {
                return StaticResult::Unknown;
            }
        }
        _ => return StaticResult::Unknown,
    };
    if arg_count == 4 {
        return StaticResult::Scalar;
    }
    match args[4] {
        Node::NumberKind(f) => {
            if f != 1.0 {
                return StaticResult::Unknown;
            }
        }
        _ => return StaticResult::Unknown,
    };
    StaticResult::Unknown
}

fn static_analysis_choose(_args: &[Node], _cell: &CellReferenceIndex) -> StaticResult {
    // We will always insert the @ in CHOOSE, but technically it is only needed if one of the elements is a range
    StaticResult::Unknown
}

fn static_analysis_indirect(_args: &[Node], _cell: &CellReferenceIndex) -> StaticResult {
    // We will always insert the @, but we don't need to do that in every scenario`
    StaticResult::Unknown
}

fn static_analysis_index(_args: &[Node], _cell: &CellReferenceIndex) -> StaticResult {
    // INDEX has two forms, but they are indistinguishable at parse time.
    StaticResult::Unknown
}

#[derive(Clone)]
enum Signature {
    Scalar,
    Vector,
    Error,
}

fn args_signature_no_args(arg_count: usize) -> Vec<Signature> {
    if arg_count == 0 {
        vec![]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_scalars(
    arg_count: usize,
    required_count: usize,
    optional_count: usize,
) -> Vec<Signature> {
    if arg_count >= required_count && arg_count <= required_count + optional_count {
        vec![Signature::Scalar; arg_count]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_one_vector(arg_count: usize) -> Vec<Signature> {
    if arg_count == 1 {
        vec![Signature::Vector]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_sumif(arg_count: usize) -> Vec<Signature> {
    if arg_count == 2 {
        vec![Signature::Vector, Signature::Scalar]
    } else if arg_count == 3 {
        vec![Signature::Vector, Signature::Scalar, Signature::Vector]
    } else {
        vec![Signature::Error; arg_count]
    }
}

// 1 or none scalars
fn args_signature_sheet(arg_count: usize) -> Vec<Signature> {
    if arg_count == 0 {
        vec![]
    } else if arg_count == 1 {
        vec![Signature::Scalar]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_hlookup(arg_count: usize) -> Vec<Signature> {
    if arg_count == 3 {
        vec![Signature::Vector, Signature::Vector, Signature::Scalar]
    } else if arg_count == 4 {
        vec![
            Signature::Vector,
            Signature::Vector,
            Signature::Scalar,
            Signature::Vector,
        ]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_index(arg_count: usize) -> Vec<Signature> {
    if arg_count == 2 {
        vec![Signature::Vector, Signature::Scalar]
    } else if arg_count == 3 {
        vec![Signature::Vector, Signature::Scalar, Signature::Scalar]
    } else if arg_count == 4 {
        vec![
            Signature::Vector,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
        ]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_lookup(arg_count: usize) -> Vec<Signature> {
    if arg_count == 2 {
        vec![Signature::Vector, Signature::Vector]
    } else if arg_count == 3 {
        vec![Signature::Vector, Signature::Vector, Signature::Vector]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_match(arg_count: usize) -> Vec<Signature> {
    if arg_count == 2 {
        vec![Signature::Vector, Signature::Vector]
    } else if arg_count == 3 {
        vec![Signature::Vector, Signature::Vector, Signature::Scalar]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_offset(arg_count: usize) -> Vec<Signature> {
    if arg_count == 3 {
        vec![Signature::Vector, Signature::Scalar, Signature::Scalar]
    } else if arg_count == 4 {
        vec![
            Signature::Vector,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
        ]
    } else if arg_count == 5 {
        vec![
            Signature::Vector,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
        ]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_row(arg_count: usize) -> Vec<Signature> {
    if arg_count == 0 {
        vec![]
    } else if arg_count == 1 {
        vec![Signature::Vector]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_xlookup(arg_count: usize) -> Vec<Signature> {
    if !(3..=6).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result[1] = Signature::Vector;
    result[2] = Signature::Vector;
    result
}

fn args_signature_textafter(arg_count: usize) -> Vec<Signature> {
    if !(2..=6).contains(&arg_count) {
        vec![Signature::Scalar; arg_count]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_textjoin(arg_count: usize) -> Vec<Signature> {
    if arg_count >= 3 {
        let mut result = vec![Signature::Vector; arg_count];
        result[0] = Signature::Scalar;
        result[1] = Signature::Scalar;
        result
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_npv(arg_count: usize) -> Vec<Signature> {
    if arg_count < 2 {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Vector; arg_count];
    result[0] = Signature::Scalar;
    result
}

fn args_signature_irr(arg_count: usize) -> Vec<Signature> {
    if arg_count > 2 {
        vec![Signature::Error; arg_count]
    } else if arg_count == 1 {
        vec![Signature::Vector]
    } else {
        vec![Signature::Vector, Signature::Scalar]
    }
}

fn args_signature_xirr(arg_count: usize) -> Vec<Signature> {
    if arg_count == 2 {
        vec![Signature::Vector; arg_count]
    } else if arg_count == 3 {
        vec![Signature::Vector, Signature::Vector, Signature::Scalar]
    } else {
        vec![Signature::Error; arg_count]
    }
}

fn args_signature_mirr(arg_count: usize) -> Vec<Signature> {
    if arg_count != 3 {
        vec![Signature::Error; arg_count]
    } else {
        vec![Signature::Vector, Signature::Scalar, Signature::Scalar]
    }
}

fn args_signature_xnpv(arg_count: usize) -> Vec<Signature> {
    if arg_count != 3 {
        vec![Signature::Error; arg_count]
    } else {
        vec![Signature::Scalar, Signature::Vector, Signature::Vector]
    }
}

// FIXME: This is terrible duplications of efforts. We use the signature in at least three different places:
// 1. When computing the function
// 2. Checking the arguments to see if we need to insert the implicit intersection operator
// 3. Understanding the return value
//
// The signature of the functions should be defined only once

// Given a function and a number of arguments this returns the arguments at each position
// are expected to be scalars or vectors (array/ranges).
// Sets signature::Error to all arguments if the number of arguments is incorrect.
fn get_function_args_signature(kind: &Function, arg_count: usize) -> Vec<Signature> {
    match kind {
        Function::And => vec![Signature::Vector; arg_count],
        Function::False => vec![Signature::Error; arg_count],
        Function::If => args_signature_scalars(arg_count, 2, 1),
        Function::Iferror => args_signature_scalars(arg_count, 2, 0),
        Function::Ifna => args_signature_scalars(arg_count, 2, 0),
        Function::Ifs => vec![Signature::Scalar; arg_count],
        Function::Not => args_signature_scalars(arg_count, 1, 0),
        Function::Or => vec![Signature::Vector; arg_count],
        Function::Switch => vec![Signature::Scalar; arg_count],
        Function::True => args_signature_no_args(arg_count),
        Function::Xor => vec![Signature::Vector; arg_count],
        Function::Abs => args_signature_scalars(arg_count, 1, 0),
        Function::Acos => args_signature_scalars(arg_count, 1, 0),
        Function::Acosh => args_signature_scalars(arg_count, 1, 0),
        Function::Asin => args_signature_scalars(arg_count, 1, 0),
        Function::Asinh => args_signature_scalars(arg_count, 1, 0),
        Function::Atan => args_signature_scalars(arg_count, 1, 0),
        Function::Atan2 => args_signature_scalars(arg_count, 2, 0),
        Function::Atanh => args_signature_scalars(arg_count, 1, 0),
        Function::Choose => vec![Signature::Scalar; arg_count],
        Function::Column => args_signature_row(arg_count),
        Function::Columns => args_signature_one_vector(arg_count),
        Function::Cos => args_signature_scalars(arg_count, 1, 0),
        Function::Cosh => args_signature_scalars(arg_count, 1, 0),
        Function::Max => vec![Signature::Vector; arg_count],
        Function::Min => vec![Signature::Vector; arg_count],
        Function::Pi => args_signature_no_args(arg_count),
        Function::Power => args_signature_scalars(arg_count, 2, 0),
        Function::Product => vec![Signature::Vector; arg_count],
        Function::Round => args_signature_scalars(arg_count, 2, 0),
        Function::Rounddown => args_signature_scalars(arg_count, 2, 0),
        Function::Roundup => args_signature_scalars(arg_count, 2, 0),
        Function::Sin => args_signature_scalars(arg_count, 1, 0),
        Function::Sinh => args_signature_scalars(arg_count, 1, 0),
        Function::Sqrt => args_signature_scalars(arg_count, 1, 0),
        Function::Sqrtpi => args_signature_no_args(arg_count),
        Function::Sum => vec![Signature::Vector; arg_count],
        Function::Sumif => args_signature_sumif(arg_count),
        Function::Sumifs => vec![Signature::Vector; arg_count],
        Function::Tan => args_signature_scalars(arg_count, 1, 0),
        Function::Tanh => args_signature_scalars(arg_count, 1, 0),
        Function::ErrorType => args_signature_scalars(arg_count, 1, 0),
        Function::Isblank => args_signature_scalars(arg_count, 1, 0),
        Function::Iserr => args_signature_scalars(arg_count, 1, 0),
        Function::Iserror => args_signature_scalars(arg_count, 1, 0),
        Function::Iseven => args_signature_scalars(arg_count, 1, 0),
        Function::Isformula => args_signature_scalars(arg_count, 1, 0),
        Function::Islogical => args_signature_scalars(arg_count, 1, 0),
        Function::Isna => args_signature_scalars(arg_count, 1, 0),
        Function::Isnontext => args_signature_scalars(arg_count, 1, 0),
        Function::Isnumber => args_signature_scalars(arg_count, 1, 0),
        Function::Isodd => args_signature_scalars(arg_count, 1, 0),
        Function::Isref => args_signature_one_vector(arg_count),
        Function::Istext => args_signature_scalars(arg_count, 1, 0),
        Function::Na => args_signature_no_args(arg_count),
        Function::Sheet => args_signature_sheet(arg_count),
        Function::Type => args_signature_one_vector(arg_count),
        Function::Hlookup => args_signature_hlookup(arg_count),
        Function::Index => args_signature_index(arg_count),
        Function::Indirect => args_signature_scalars(arg_count, 1, 0),
        Function::Lookup => args_signature_lookup(arg_count),
        Function::Match => args_signature_match(arg_count),
        Function::Offset => args_signature_offset(arg_count),
        Function::Row => args_signature_row(arg_count),
        Function::Rows => args_signature_one_vector(arg_count),
        Function::Vlookup => args_signature_hlookup(arg_count),
        Function::Xlookup => args_signature_xlookup(arg_count),
        Function::Concat => vec![Signature::Vector; arg_count],
        Function::Concatenate => vec![Signature::Scalar; arg_count],
        Function::Exact => args_signature_scalars(arg_count, 2, 0),
        Function::Find => args_signature_scalars(arg_count, 2, 1),
        Function::Left => args_signature_scalars(arg_count, 1, 1),
        Function::Len => args_signature_scalars(arg_count, 1, 0),
        Function::Lower => args_signature_scalars(arg_count, 1, 0),
        Function::Mid => args_signature_scalars(arg_count, 3, 0),
        Function::Rept => args_signature_scalars(arg_count, 2, 0),
        Function::Right => args_signature_scalars(arg_count, 2, 1),
        Function::Search => args_signature_scalars(arg_count, 2, 1),
        Function::Substitute => args_signature_scalars(arg_count, 3, 1),
        Function::T => args_signature_scalars(arg_count, 1, 0),
        Function::Text => args_signature_scalars(arg_count, 2, 0),
        Function::Textafter => args_signature_textafter(arg_count),
        Function::Textbefore => args_signature_textafter(arg_count),
        Function::Textjoin => args_signature_textjoin(arg_count),
        Function::Trim => args_signature_scalars(arg_count, 1, 0),
        Function::Upper => args_signature_scalars(arg_count, 1, 0),
        Function::Value => args_signature_scalars(arg_count, 1, 0),
        Function::Valuetotext => args_signature_scalars(arg_count, 1, 1),
        Function::Average => vec![Signature::Vector; arg_count],
        Function::Averagea => vec![Signature::Vector; arg_count],
        Function::Averageif => args_signature_sumif(arg_count),
        Function::Averageifs => vec![Signature::Vector; arg_count],
        Function::Count => vec![Signature::Vector; arg_count],
        Function::Counta => vec![Signature::Vector; arg_count],
        Function::Countblank => vec![Signature::Vector; arg_count],
        Function::Countif => args_signature_sumif(arg_count),
        Function::Countifs => vec![Signature::Vector; arg_count],
        Function::Maxifs => vec![Signature::Vector; arg_count],
        Function::Minifs => vec![Signature::Vector; arg_count],
        Function::Date => args_signature_scalars(arg_count, 3, 0),
        Function::Day => args_signature_scalars(arg_count, 1, 0),
        Function::Edate => args_signature_scalars(arg_count, 2, 0),
        Function::Eomonth => args_signature_scalars(arg_count, 2, 0),
        Function::Month => args_signature_scalars(arg_count, 1, 0),
        Function::Now => args_signature_no_args(arg_count),
        Function::Today => args_signature_no_args(arg_count),
        Function::Year => args_signature_scalars(arg_count, 1, 0),
        Function::Cumipmt => args_signature_scalars(arg_count, 6, 0),
        Function::Cumprinc => args_signature_scalars(arg_count, 6, 0),
        Function::Db => args_signature_scalars(arg_count, 4, 1),
        Function::Ddb => args_signature_scalars(arg_count, 4, 1),
        Function::Dollarde => args_signature_scalars(arg_count, 2, 0),
        Function::Dollarfr => args_signature_scalars(arg_count, 2, 0),
        Function::Effect => args_signature_scalars(arg_count, 2, 0),
        Function::Fv => args_signature_scalars(arg_count, 3, 2),
        Function::Ipmt => args_signature_scalars(arg_count, 4, 2),
        Function::Irr => args_signature_irr(arg_count),
        Function::Ispmt => args_signature_scalars(arg_count, 4, 0),
        Function::Mirr => args_signature_mirr(arg_count),
        Function::Nominal => args_signature_scalars(arg_count, 2, 0),
        Function::Nper => args_signature_scalars(arg_count, 3, 2),
        Function::Npv => args_signature_npv(arg_count),
        Function::Pduration => args_signature_scalars(arg_count, 3, 0),
        Function::Pmt => args_signature_scalars(arg_count, 3, 2),
        Function::Ppmt => args_signature_scalars(arg_count, 4, 2),
        Function::Pv => args_signature_scalars(arg_count, 3, 2),
        Function::Rate => args_signature_scalars(arg_count, 3, 3),
        Function::Rri => args_signature_scalars(arg_count, 3, 0),
        Function::Sln => args_signature_scalars(arg_count, 3, 0),
        Function::Syd => args_signature_scalars(arg_count, 4, 0),
        Function::Tbilleq => args_signature_scalars(arg_count, 3, 0),
        Function::Tbillprice => args_signature_scalars(arg_count, 3, 0),
        Function::Tbillyield => args_signature_scalars(arg_count, 3, 0),
        Function::Xirr => args_signature_xirr(arg_count),
        Function::Xnpv => args_signature_xnpv(arg_count),
        Function::Besseli => args_signature_scalars(arg_count, 2, 0),
        Function::Besselj => args_signature_scalars(arg_count, 2, 0),
        Function::Besselk => args_signature_scalars(arg_count, 2, 0),
        Function::Bessely => args_signature_scalars(arg_count, 2, 0),
        Function::Erf => args_signature_scalars(arg_count, 1, 1),
        Function::Erfc => args_signature_scalars(arg_count, 1, 0),
        Function::ErfcPrecise => args_signature_scalars(arg_count, 1, 0),
        Function::ErfPrecise => args_signature_scalars(arg_count, 1, 0),
        Function::Bin2dec => args_signature_scalars(arg_count, 1, 0),
        Function::Bin2hex => args_signature_scalars(arg_count, 1, 0),
        Function::Bin2oct => args_signature_scalars(arg_count, 1, 0),
        Function::Dec2Bin => args_signature_scalars(arg_count, 1, 0),
        Function::Dec2hex => args_signature_scalars(arg_count, 1, 0),
        Function::Dec2oct => args_signature_scalars(arg_count, 1, 0),
        Function::Hex2bin => args_signature_scalars(arg_count, 1, 0),
        Function::Hex2dec => args_signature_scalars(arg_count, 1, 0),
        Function::Hex2oct => args_signature_scalars(arg_count, 1, 0),
        Function::Oct2bin => args_signature_scalars(arg_count, 1, 0),
        Function::Oct2dec => args_signature_scalars(arg_count, 1, 0),
        Function::Oct2hex => args_signature_scalars(arg_count, 1, 0),
        Function::Bitand => args_signature_scalars(arg_count, 2, 0),
        Function::Bitlshift => args_signature_scalars(arg_count, 2, 0),
        Function::Bitor => args_signature_scalars(arg_count, 2, 0),
        Function::Bitrshift => args_signature_scalars(arg_count, 2, 0),
        Function::Bitxor => args_signature_scalars(arg_count, 2, 0),
        Function::Complex => args_signature_scalars(arg_count, 2, 1),
        Function::Imabs => args_signature_scalars(arg_count, 1, 0),
        Function::Imaginary => args_signature_scalars(arg_count, 1, 0),
        Function::Imargument => args_signature_scalars(arg_count, 1, 0),
        Function::Imconjugate => args_signature_scalars(arg_count, 1, 0),
        Function::Imcos => args_signature_scalars(arg_count, 1, 0),
        Function::Imcosh => args_signature_scalars(arg_count, 1, 0),
        Function::Imcot => args_signature_scalars(arg_count, 1, 0),
        Function::Imcsc => args_signature_scalars(arg_count, 1, 0),
        Function::Imcsch => args_signature_scalars(arg_count, 1, 0),
        Function::Imdiv => args_signature_scalars(arg_count, 2, 0),
        Function::Imexp => args_signature_scalars(arg_count, 1, 0),
        Function::Imln => args_signature_scalars(arg_count, 1, 0),
        Function::Imlog10 => args_signature_scalars(arg_count, 1, 0),
        Function::Imlog2 => args_signature_scalars(arg_count, 1, 0),
        Function::Impower => args_signature_scalars(arg_count, 2, 0),
        Function::Improduct => args_signature_scalars(arg_count, 2, 0),
        Function::Imreal => args_signature_scalars(arg_count, 1, 0),
        Function::Imsec => args_signature_scalars(arg_count, 1, 0),
        Function::Imsech => args_signature_scalars(arg_count, 1, 0),
        Function::Imsin => args_signature_scalars(arg_count, 1, 0),
        Function::Imsinh => args_signature_scalars(arg_count, 1, 0),
        Function::Imsqrt => args_signature_scalars(arg_count, 1, 0),
        Function::Imsub => args_signature_scalars(arg_count, 2, 0),
        Function::Imsum => args_signature_scalars(arg_count, 2, 0),
        Function::Imtan => args_signature_scalars(arg_count, 1, 0),
        Function::Convert => args_signature_scalars(arg_count, 3, 0),
        Function::Delta => args_signature_scalars(arg_count, 1, 1),
        Function::Gestep => args_signature_scalars(arg_count, 1, 1),
        Function::Subtotal => args_signature_npv(arg_count),
        Function::Rand => args_signature_no_args(arg_count),
        Function::Randbetween => args_signature_scalars(arg_count, 2, 0),
        Function::Formulatext => args_signature_scalars(arg_count, 1, 0),
        Function::Unicode => args_signature_scalars(arg_count, 1, 0),
        Function::Geomean => todo!(),
    }
}

// Returns the type of the result (Scalar, Array or Range) depending on the arguments
fn static_analysis_on_function(
    kind: &Function,
    args: &[Node],
    cell: &CellReferenceIndex,
) -> StaticResult {
    match kind {
        Function::And => StaticResult::Scalar,
        Function::False => StaticResult::Scalar,
        Function::If => scalar_arguments(args, cell),
        Function::Iferror => scalar_arguments(args, cell),
        Function::Ifna => scalar_arguments(args, cell),
        Function::Ifs => not_implemented(args, cell),
        Function::Not => StaticResult::Scalar,
        Function::Or => StaticResult::Scalar,
        Function::Switch => not_implemented(args, cell),
        Function::True => StaticResult::Scalar,
        Function::Xor => StaticResult::Scalar,
        Function::Abs => scalar_arguments(args, cell),
        Function::Acos => scalar_arguments(args, cell),
        Function::Acosh => scalar_arguments(args, cell),
        Function::Asin => scalar_arguments(args, cell),
        Function::Asinh => scalar_arguments(args, cell),
        Function::Atan => scalar_arguments(args, cell),
        Function::Atan2 => scalar_arguments(args, cell),
        Function::Atanh => scalar_arguments(args, cell),
        Function::Choose => scalar_arguments(args, cell), // static_analysis_choose(args, cell),
        Function::Column => not_implemented(args, cell),
        Function::Columns => not_implemented(args, cell),
        Function::Cos => scalar_arguments(args, cell),
        Function::Cosh => scalar_arguments(args, cell),
        Function::Max => StaticResult::Scalar,
        Function::Min => StaticResult::Scalar,
        Function::Pi => StaticResult::Scalar,
        Function::Power => scalar_arguments(args, cell),
        Function::Product => not_implemented(args, cell),
        Function::Round => scalar_arguments(args, cell),
        Function::Rounddown => scalar_arguments(args, cell),
        Function::Roundup => scalar_arguments(args, cell),
        Function::Sin => scalar_arguments(args, cell),
        Function::Sinh => scalar_arguments(args, cell),
        Function::Sqrt => scalar_arguments(args, cell),
        Function::Sqrtpi => StaticResult::Scalar,
        Function::Sum => StaticResult::Scalar,
        Function::Sumif => not_implemented(args, cell),
        Function::Sumifs => not_implemented(args, cell),
        Function::Tan => scalar_arguments(args, cell),
        Function::Tanh => scalar_arguments(args, cell),
        Function::ErrorType => not_implemented(args, cell),
        Function::Isblank => not_implemented(args, cell),
        Function::Iserr => not_implemented(args, cell),
        Function::Iserror => not_implemented(args, cell),
        Function::Iseven => not_implemented(args, cell),
        Function::Isformula => not_implemented(args, cell),
        Function::Islogical => not_implemented(args, cell),
        Function::Isna => not_implemented(args, cell),
        Function::Isnontext => not_implemented(args, cell),
        Function::Isnumber => not_implemented(args, cell),
        Function::Isodd => not_implemented(args, cell),
        Function::Isref => not_implemented(args, cell),
        Function::Istext => not_implemented(args, cell),
        Function::Na => StaticResult::Scalar,
        Function::Sheet => StaticResult::Scalar,
        Function::Type => not_implemented(args, cell),
        Function::Hlookup => not_implemented(args, cell),
        Function::Index => static_analysis_index(args, cell),
        Function::Indirect => static_analysis_indirect(args, cell),
        Function::Lookup => not_implemented(args, cell),
        Function::Match => not_implemented(args, cell),
        Function::Offset => static_analysis_offset(args, cell),
        // FIXME: Row could return an array
        Function::Row => StaticResult::Scalar,
        Function::Rows => not_implemented(args, cell),
        Function::Vlookup => not_implemented(args, cell),
        Function::Xlookup => not_implemented(args, cell),
        Function::Concat => not_implemented(args, cell),
        Function::Concatenate => not_implemented(args, cell),
        Function::Exact => not_implemented(args, cell),
        Function::Find => not_implemented(args, cell),
        Function::Left => not_implemented(args, cell),
        Function::Len => not_implemented(args, cell),
        Function::Lower => not_implemented(args, cell),
        Function::Mid => not_implemented(args, cell),
        Function::Rept => not_implemented(args, cell),
        Function::Right => not_implemented(args, cell),
        Function::Search => not_implemented(args, cell),
        Function::Substitute => not_implemented(args, cell),
        Function::T => not_implemented(args, cell),
        Function::Text => not_implemented(args, cell),
        Function::Textafter => not_implemented(args, cell),
        Function::Textbefore => not_implemented(args, cell),
        Function::Textjoin => not_implemented(args, cell),
        Function::Trim => not_implemented(args, cell),
        Function::Unicode => not_implemented(args, cell),
        Function::Upper => not_implemented(args, cell),
        Function::Value => not_implemented(args, cell),
        Function::Valuetotext => not_implemented(args, cell),
        Function::Average => not_implemented(args, cell),
        Function::Averagea => not_implemented(args, cell),
        Function::Averageif => not_implemented(args, cell),
        Function::Averageifs => not_implemented(args, cell),
        Function::Count => not_implemented(args, cell),
        Function::Counta => not_implemented(args, cell),
        Function::Countblank => not_implemented(args, cell),
        Function::Countif => not_implemented(args, cell),
        Function::Countifs => not_implemented(args, cell),
        Function::Maxifs => not_implemented(args, cell),
        Function::Minifs => not_implemented(args, cell),
        Function::Date => not_implemented(args, cell),
        Function::Day => not_implemented(args, cell),
        Function::Edate => not_implemented(args, cell),
        Function::Month => not_implemented(args, cell),
        Function::Now => not_implemented(args, cell),
        Function::Today => not_implemented(args, cell),
        Function::Year => not_implemented(args, cell),
        Function::Cumipmt => not_implemented(args, cell),
        Function::Cumprinc => not_implemented(args, cell),
        Function::Db => not_implemented(args, cell),
        Function::Ddb => not_implemented(args, cell),
        Function::Dollarde => not_implemented(args, cell),
        Function::Dollarfr => not_implemented(args, cell),
        Function::Effect => not_implemented(args, cell),
        Function::Fv => not_implemented(args, cell),
        Function::Ipmt => not_implemented(args, cell),
        Function::Irr => not_implemented(args, cell),
        Function::Ispmt => not_implemented(args, cell),
        Function::Mirr => not_implemented(args, cell),
        Function::Nominal => not_implemented(args, cell),
        Function::Nper => not_implemented(args, cell),
        Function::Npv => not_implemented(args, cell),
        Function::Pduration => not_implemented(args, cell),
        Function::Pmt => not_implemented(args, cell),
        Function::Ppmt => not_implemented(args, cell),
        Function::Pv => not_implemented(args, cell),
        Function::Rate => not_implemented(args, cell),
        Function::Rri => not_implemented(args, cell),
        Function::Sln => not_implemented(args, cell),
        Function::Syd => not_implemented(args, cell),
        Function::Tbilleq => not_implemented(args, cell),
        Function::Tbillprice => not_implemented(args, cell),
        Function::Tbillyield => not_implemented(args, cell),
        Function::Xirr => not_implemented(args, cell),
        Function::Xnpv => not_implemented(args, cell),
        Function::Besseli => scalar_arguments(args, cell),
        Function::Besselj => scalar_arguments(args, cell),
        Function::Besselk => scalar_arguments(args, cell),
        Function::Bessely => scalar_arguments(args, cell),
        Function::Erf => scalar_arguments(args, cell),
        Function::Erfc => scalar_arguments(args, cell),
        Function::ErfcPrecise => scalar_arguments(args, cell),
        Function::ErfPrecise => scalar_arguments(args, cell),
        Function::Bin2dec => scalar_arguments(args, cell),
        Function::Bin2hex => scalar_arguments(args, cell),
        Function::Bin2oct => scalar_arguments(args, cell),
        Function::Dec2Bin => scalar_arguments(args, cell),
        Function::Dec2hex => scalar_arguments(args, cell),
        Function::Dec2oct => scalar_arguments(args, cell),
        Function::Hex2bin => scalar_arguments(args, cell),
        Function::Hex2dec => scalar_arguments(args, cell),
        Function::Hex2oct => scalar_arguments(args, cell),
        Function::Oct2bin => scalar_arguments(args, cell),
        Function::Oct2dec => scalar_arguments(args, cell),
        Function::Oct2hex => scalar_arguments(args, cell),
        Function::Bitand => scalar_arguments(args, cell),
        Function::Bitlshift => scalar_arguments(args, cell),
        Function::Bitor => scalar_arguments(args, cell),
        Function::Bitrshift => scalar_arguments(args, cell),
        Function::Bitxor => scalar_arguments(args, cell),
        Function::Complex => scalar_arguments(args, cell),
        Function::Imabs => scalar_arguments(args, cell),
        Function::Imaginary => scalar_arguments(args, cell),
        Function::Imargument => scalar_arguments(args, cell),
        Function::Imconjugate => scalar_arguments(args, cell),
        Function::Imcos => scalar_arguments(args, cell),
        Function::Imcosh => scalar_arguments(args, cell),
        Function::Imcot => scalar_arguments(args, cell),
        Function::Imcsc => scalar_arguments(args, cell),
        Function::Imcsch => scalar_arguments(args, cell),
        Function::Imdiv => scalar_arguments(args, cell),
        Function::Imexp => scalar_arguments(args, cell),
        Function::Imln => scalar_arguments(args, cell),
        Function::Imlog10 => scalar_arguments(args, cell),
        Function::Imlog2 => scalar_arguments(args, cell),
        Function::Impower => scalar_arguments(args, cell),
        Function::Improduct => scalar_arguments(args, cell),
        Function::Imreal => scalar_arguments(args, cell),
        Function::Imsec => scalar_arguments(args, cell),
        Function::Imsech => scalar_arguments(args, cell),
        Function::Imsin => scalar_arguments(args, cell),
        Function::Imsinh => scalar_arguments(args, cell),
        Function::Imsqrt => scalar_arguments(args, cell),
        Function::Imsub => scalar_arguments(args, cell),
        Function::Imsum => scalar_arguments(args, cell),
        Function::Imtan => scalar_arguments(args, cell),
        Function::Convert => not_implemented(args, cell),
        Function::Delta => not_implemented(args, cell),
        Function::Gestep => not_implemented(args, cell),
        Function::Subtotal => not_implemented(args, cell),
        Function::Rand => not_implemented(args, cell),
        Function::Randbetween => scalar_arguments(args, cell),
        Function::Eomonth => scalar_arguments(args, cell),
        Function::Formulatext => not_implemented(args, cell),
        Function::Geomean => todo!(),
    }
}
