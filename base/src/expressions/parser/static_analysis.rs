use crate::functions::Function;

use super::Node;

use once_cell::sync::Lazy;
use regex::Regex;

#[allow(clippy::expect_used)]
static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r":[A-Z]*[0-9]*$").expect("Regex is known to be valid"));

fn is_range_reference(s: &str) -> bool {
    RE.is_match(s)
}

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
pub fn add_implicit_intersection(node: &mut Node, add: bool) {
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
            add_implicit_intersection(&mut new_node, add);
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
        Node::UnaryKind { right, .. } => add_implicit_intersection(right, add),
        Node::OpConcatenateKind { left, right }
        | Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right, .. }
        | Node::CompareKind { left, right, .. } => {
            add_implicit_intersection(left, add);
            add_implicit_intersection(right, add);
        }

        Node::DefinedNameKind(v) => {
            if add {
                // Not all defined names deserve the II operator
                // For instance =Sheet1!A1 doesn't need to be intersected
                if is_range_reference(&v.2) {
                    *node = Node::ImplicitIntersection {
                        automatic: true,
                        child: Box::new(Node::DefinedNameKind(v.to_owned())),
                    }
                }
            }
        }
        Node::WrongVariableKind(v) => {
            if add {
                *node = Node::ImplicitIntersection {
                    automatic: true,
                    child: Box::new(Node::WrongVariableKind(v.to_owned())),
                }
            }
        }
        Node::TableNameKind(_) => {
            // noop for now
        }
        Node::FunctionKind { kind, args } => {
            let arg_count = args.len();
            let signature = get_function_args_signature(kind, arg_count);
            for index in 0..arg_count {
                if matches!(signature[index], Signature::Scalar)
                    && matches!(
                        run_static_analysis_on_node(&args[index]),
                        StaticResult::Range(_, _) | StaticResult::Unknown
                    )
                {
                    add_implicit_intersection(&mut args[index], true);
                } else {
                    add_implicit_intersection(&mut args[index], false);
                }
            }
            if add
                && matches!(
                    run_static_analysis_on_node(node),
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

fn static_analysis_op_nodes(left: &Node, right: &Node) -> StaticResult {
    let lhs = run_static_analysis_on_node(left);
    let rhs = run_static_analysis_on_node(right);
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
fn run_static_analysis_on_node(node: &Node) -> StaticResult {
    match node {
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::EmptyArgKind => StaticResult::Scalar,
        Node::UnaryKind { right, .. } => run_static_analysis_on_node(right),
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
            let n = array.len() as i32;
            // FIXME: This is a placeholder until we implement arrays
            StaticResult::Array(n, 1)
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
        Node::OpConcatenateKind { left, right } => static_analysis_op_nodes(left, right),
        Node::OpSumKind { left, right, .. } => static_analysis_op_nodes(left, right),
        Node::OpProductKind { left, right, .. } => static_analysis_op_nodes(left, right),
        Node::OpPowerKind { left, right, .. } => static_analysis_op_nodes(left, right),
        Node::CompareKind { left, right, .. } => static_analysis_op_nodes(left, right),

        // defined names
        Node::DefinedNameKind(_) => StaticResult::Unknown,
        Node::WrongVariableKind(_) => StaticResult::Unknown,
        Node::TableNameKind(_) => StaticResult::Unknown,
        Node::FunctionKind { kind, args } => static_analysis_on_function(kind, args),
        Node::ImplicitIntersection { .. } => StaticResult::Scalar,
    }
}

// If all the arguments are scalars the function will return a scalar
// If any of the arguments is a range or an array it will return an array
fn scalar_arguments(args: &[Node]) -> StaticResult {
    let mut n = 0;
    let mut m = 0;
    for arg in args {
        match run_static_analysis_on_node(arg) {
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
fn not_implemented(_args: &[Node]) -> StaticResult {
    StaticResult::Scalar
}

fn static_analysis_offset(args: &[Node]) -> StaticResult {
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

// fn static_analysis_choose(_args: &[Node]) -> StaticResult {
//     // We will always insert the @ in CHOOSE, but technically it is only needed if one of the elements is a range
//     StaticResult::Unknown
// }

fn static_analysis_indirect(_args: &[Node]) -> StaticResult {
    // We will always insert the @, but we don't need to do that in every scenario`
    StaticResult::Unknown
}

fn static_analysis_index(_args: &[Node]) -> StaticResult {
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
        Function::False => args_signature_no_args(arg_count),
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
        Function::Sqrtpi => args_signature_scalars(arg_count, 1, 0),
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
        Function::Geomean => vec![Signature::Vector; arg_count],
    }
}

// Returns the type of the result (Scalar, Array or Range) depending on the arguments
fn static_analysis_on_function(kind: &Function, args: &[Node]) -> StaticResult {
    match kind {
        Function::And => StaticResult::Scalar,
        Function::False => StaticResult::Scalar,
        Function::If => scalar_arguments(args),
        Function::Iferror => scalar_arguments(args),
        Function::Ifna => scalar_arguments(args),
        Function::Ifs => not_implemented(args),
        Function::Not => StaticResult::Scalar,
        Function::Or => StaticResult::Scalar,
        Function::Switch => not_implemented(args),
        Function::True => StaticResult::Scalar,
        Function::Xor => StaticResult::Scalar,
        Function::Abs => scalar_arguments(args),
        Function::Acos => scalar_arguments(args),
        Function::Acosh => scalar_arguments(args),
        Function::Asin => scalar_arguments(args),
        Function::Asinh => scalar_arguments(args),
        Function::Atan => scalar_arguments(args),
        Function::Atan2 => scalar_arguments(args),
        Function::Atanh => scalar_arguments(args),
        Function::Choose => scalar_arguments(args), // static_analysis_choose(args, cell),
        Function::Column => not_implemented(args),
        Function::Columns => not_implemented(args),
        Function::Cos => scalar_arguments(args),
        Function::Cosh => scalar_arguments(args),
        Function::Max => StaticResult::Scalar,
        Function::Min => StaticResult::Scalar,
        Function::Pi => StaticResult::Scalar,
        Function::Power => scalar_arguments(args),
        Function::Product => not_implemented(args),
        Function::Round => scalar_arguments(args),
        Function::Rounddown => scalar_arguments(args),
        Function::Roundup => scalar_arguments(args),
        Function::Sin => scalar_arguments(args),
        Function::Sinh => scalar_arguments(args),
        Function::Sqrt => scalar_arguments(args),
        Function::Sqrtpi => StaticResult::Scalar,
        Function::Sum => StaticResult::Scalar,
        Function::Sumif => not_implemented(args),
        Function::Sumifs => not_implemented(args),
        Function::Tan => scalar_arguments(args),
        Function::Tanh => scalar_arguments(args),
        Function::ErrorType => not_implemented(args),
        Function::Isblank => not_implemented(args),
        Function::Iserr => not_implemented(args),
        Function::Iserror => not_implemented(args),
        Function::Iseven => not_implemented(args),
        Function::Isformula => not_implemented(args),
        Function::Islogical => not_implemented(args),
        Function::Isna => not_implemented(args),
        Function::Isnontext => not_implemented(args),
        Function::Isnumber => not_implemented(args),
        Function::Isodd => not_implemented(args),
        Function::Isref => not_implemented(args),
        Function::Istext => not_implemented(args),
        Function::Na => StaticResult::Scalar,
        Function::Sheet => StaticResult::Scalar,
        Function::Type => not_implemented(args),
        Function::Hlookup => not_implemented(args),
        Function::Index => static_analysis_index(args),
        Function::Indirect => static_analysis_indirect(args),
        Function::Lookup => not_implemented(args),
        Function::Match => not_implemented(args),
        Function::Offset => static_analysis_offset(args),
        // FIXME: Row could return an array
        Function::Row => StaticResult::Scalar,
        Function::Rows => not_implemented(args),
        Function::Vlookup => not_implemented(args),
        Function::Xlookup => not_implemented(args),
        Function::Concat => not_implemented(args),
        Function::Concatenate => not_implemented(args),
        Function::Exact => not_implemented(args),
        Function::Find => not_implemented(args),
        Function::Left => not_implemented(args),
        Function::Len => not_implemented(args),
        Function::Lower => not_implemented(args),
        Function::Mid => not_implemented(args),
        Function::Rept => not_implemented(args),
        Function::Right => not_implemented(args),
        Function::Search => not_implemented(args),
        Function::Substitute => not_implemented(args),
        Function::T => not_implemented(args),
        Function::Text => not_implemented(args),
        Function::Textafter => not_implemented(args),
        Function::Textbefore => not_implemented(args),
        Function::Textjoin => not_implemented(args),
        Function::Trim => not_implemented(args),
        Function::Unicode => not_implemented(args),
        Function::Upper => not_implemented(args),
        Function::Value => not_implemented(args),
        Function::Valuetotext => not_implemented(args),
        Function::Average => not_implemented(args),
        Function::Averagea => not_implemented(args),
        Function::Averageif => not_implemented(args),
        Function::Averageifs => not_implemented(args),
        Function::Count => not_implemented(args),
        Function::Counta => not_implemented(args),
        Function::Countblank => not_implemented(args),
        Function::Countif => not_implemented(args),
        Function::Countifs => not_implemented(args),
        Function::Maxifs => not_implemented(args),
        Function::Minifs => not_implemented(args),
        Function::Date => not_implemented(args),
        Function::Day => not_implemented(args),
        Function::Edate => not_implemented(args),
        Function::Month => not_implemented(args),
        Function::Now => not_implemented(args),
        Function::Today => not_implemented(args),
        Function::Year => not_implemented(args),
        Function::Cumipmt => not_implemented(args),
        Function::Cumprinc => not_implemented(args),
        Function::Db => not_implemented(args),
        Function::Ddb => not_implemented(args),
        Function::Dollarde => not_implemented(args),
        Function::Dollarfr => not_implemented(args),
        Function::Effect => not_implemented(args),
        Function::Fv => not_implemented(args),
        Function::Ipmt => not_implemented(args),
        Function::Irr => not_implemented(args),
        Function::Ispmt => not_implemented(args),
        Function::Mirr => not_implemented(args),
        Function::Nominal => not_implemented(args),
        Function::Nper => not_implemented(args),
        Function::Npv => not_implemented(args),
        Function::Pduration => not_implemented(args),
        Function::Pmt => not_implemented(args),
        Function::Ppmt => not_implemented(args),
        Function::Pv => not_implemented(args),
        Function::Rate => not_implemented(args),
        Function::Rri => not_implemented(args),
        Function::Sln => not_implemented(args),
        Function::Syd => not_implemented(args),
        Function::Tbilleq => not_implemented(args),
        Function::Tbillprice => not_implemented(args),
        Function::Tbillyield => not_implemented(args),
        Function::Xirr => not_implemented(args),
        Function::Xnpv => not_implemented(args),
        Function::Besseli => scalar_arguments(args),
        Function::Besselj => scalar_arguments(args),
        Function::Besselk => scalar_arguments(args),
        Function::Bessely => scalar_arguments(args),
        Function::Erf => scalar_arguments(args),
        Function::Erfc => scalar_arguments(args),
        Function::ErfcPrecise => scalar_arguments(args),
        Function::ErfPrecise => scalar_arguments(args),
        Function::Bin2dec => scalar_arguments(args),
        Function::Bin2hex => scalar_arguments(args),
        Function::Bin2oct => scalar_arguments(args),
        Function::Dec2Bin => scalar_arguments(args),
        Function::Dec2hex => scalar_arguments(args),
        Function::Dec2oct => scalar_arguments(args),
        Function::Hex2bin => scalar_arguments(args),
        Function::Hex2dec => scalar_arguments(args),
        Function::Hex2oct => scalar_arguments(args),
        Function::Oct2bin => scalar_arguments(args),
        Function::Oct2dec => scalar_arguments(args),
        Function::Oct2hex => scalar_arguments(args),
        Function::Bitand => scalar_arguments(args),
        Function::Bitlshift => scalar_arguments(args),
        Function::Bitor => scalar_arguments(args),
        Function::Bitrshift => scalar_arguments(args),
        Function::Bitxor => scalar_arguments(args),
        Function::Complex => scalar_arguments(args),
        Function::Imabs => scalar_arguments(args),
        Function::Imaginary => scalar_arguments(args),
        Function::Imargument => scalar_arguments(args),
        Function::Imconjugate => scalar_arguments(args),
        Function::Imcos => scalar_arguments(args),
        Function::Imcosh => scalar_arguments(args),
        Function::Imcot => scalar_arguments(args),
        Function::Imcsc => scalar_arguments(args),
        Function::Imcsch => scalar_arguments(args),
        Function::Imdiv => scalar_arguments(args),
        Function::Imexp => scalar_arguments(args),
        Function::Imln => scalar_arguments(args),
        Function::Imlog10 => scalar_arguments(args),
        Function::Imlog2 => scalar_arguments(args),
        Function::Impower => scalar_arguments(args),
        Function::Improduct => scalar_arguments(args),
        Function::Imreal => scalar_arguments(args),
        Function::Imsec => scalar_arguments(args),
        Function::Imsech => scalar_arguments(args),
        Function::Imsin => scalar_arguments(args),
        Function::Imsinh => scalar_arguments(args),
        Function::Imsqrt => scalar_arguments(args),
        Function::Imsub => scalar_arguments(args),
        Function::Imsum => scalar_arguments(args),
        Function::Imtan => scalar_arguments(args),
        Function::Convert => not_implemented(args),
        Function::Delta => not_implemented(args),
        Function::Gestep => not_implemented(args),
        Function::Subtotal => not_implemented(args),
        Function::Rand => not_implemented(args),
        Function::Randbetween => scalar_arguments(args),
        Function::Eomonth => scalar_arguments(args),
        Function::Formulatext => not_implemented(args),
        Function::Geomean => not_implemented(args),
    }
}
