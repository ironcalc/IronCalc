use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{spill_functions::transpose_array, Function},
    model::Model,
};

/// A matrix of numbers and nothing else, in one vector. The weights of a
/// model are millions of numbers, and held one by one as `ArrayNode`s in a
/// vector per row they take several times the room and the time.
struct Numbers {
    /// Row by row, `columns` to a row.
    values: Vec<f64>,
    rows: usize,
    columns: usize,
    /// The matrix is the transpose of what `values` holds. Transposing is
    /// then free, and the product with a transposed matrix, as common as
    /// `MMULT(a, TRANSPOSE(b))`, reads both of them row by row.
    transposed: bool,
}

impl Numbers {
    /// Rows and columns of the matrix.
    fn shape(&self) -> (usize, usize) {
        if self.transposed {
            (self.columns, self.rows)
        } else {
            (self.rows, self.columns)
        }
    }

    /// The matrix row by row.
    fn into_rows(self) -> Vec<f64> {
        if !self.transposed {
            return self.values;
        }
        let mut rows = Vec::with_capacity(self.values.len());
        for column in 0..self.columns {
            for row in 0..self.rows {
                rows.push(self.values[row * self.columns + column]);
            }
        }
        rows
    }

    fn into_array(self) -> Vec<Vec<ArrayNode>> {
        let (_, columns) = self.shape();
        self.into_rows()
            .chunks(columns)
            .map(|row| row.iter().map(|value| ArrayNode::Number(*value)).collect())
            .collect()
    }
}

/// An argument of MMULT.
enum Matrix {
    Numbers(Numbers),
    /// Anything else: an array with something that is not a number in it, or
    /// with nothing in it. MMULT then has an error to report, and which one
    /// depends on what there is and where.
    Array(Vec<Vec<ArrayNode>>),
}

impl Matrix {
    fn from_array(array: Vec<Vec<ArrayNode>>) -> Matrix {
        let rows = array.len();
        let columns = array.first().map_or(0, |row| row.len());
        let only_numbers = columns > 0
            && array.iter().all(|row| {
                row.len() == columns && row.iter().all(|v| matches!(v, ArrayNode::Number(_)))
            });
        if !only_numbers {
            return Matrix::Array(array);
        }
        let mut values = Vec::with_capacity(rows * columns);
        for value in array.iter().flatten() {
            if let ArrayNode::Number(number) = value {
                values.push(*number);
            }
        }
        Matrix::Numbers(Numbers {
            values,
            rows,
            columns,
            transposed: false,
        })
    }

    fn transpose(self) -> Matrix {
        match self {
            Matrix::Numbers(numbers) => Matrix::Numbers(Numbers {
                transposed: !numbers.transposed,
                ..numbers
            }),
            Matrix::Array(array) => Matrix::Array(transpose_array(array)),
        }
    }

    fn into_array(self) -> Vec<Vec<ArrayNode>> {
        match self {
            Matrix::Numbers(numbers) => numbers.into_array(),
            Matrix::Array(array) => array,
        }
    }
}

/// The product of two matrices of numbers.
fn multiply(a: Numbers, b: Numbers, cell: CellReferenceIndex) -> CalcResult {
    let (m, k) = a.shape();
    let (k2, n) = b.shape();
    if k != k2 {
        return CalcResult::new_error(Error::VALUE, cell, "MMULT dimension mismatch".to_string());
    }
    let a = a.into_rows();
    // C[i][j] = sum over p of A[i][p] * B[p][j], and for every element the
    // terms are added in that order, whichever way the matrices are walked.
    let mut product = vec![0.0f64; m * n];
    for (a_row, product_row) in a.chunks(k).zip(product.chunks_mut(n)) {
        if b.transposed {
            // The columns of B are the rows of what it holds
            for (b_column, sum) in b.values.chunks(k).zip(product_row.iter_mut()) {
                for (x, y) in a_row.iter().zip(b_column) {
                    *sum += x * y;
                }
            }
        } else {
            for (x, b_row) in a_row.iter().zip(b.values.chunks(n)) {
                for (sum, y) in product_row.iter_mut().zip(b_row) {
                    *sum += x * y;
                }
            }
        }
    }
    if product.iter().any(|sum| !sum.is_finite()) {
        return CalcResult::new_error(Error::NUM, cell, "MMULT result overflow".to_string());
    }
    CalcResult::Array(
        product
            .chunks(n)
            .map(|row| row.iter().map(|sum| ArrayNode::Number(*sum)).collect())
            .collect(),
    )
}

impl<'a> Model<'a> {
    // ── MMULT ─────────────────────────────────────────────────────────────────

    /// `=MMULT(array1, array2)`
    ///
    /// Returns the matrix product of two arrays. The number of columns of
    /// `array1` must equal the number of rows of `array2`. All entries must
    /// be numeric. Empty cells, booleans, and strings in either argument yield
    /// `#VALUE!`. Errors are propagated. Overflow yields `#NUM!`.
    pub(crate) fn fn_mmult(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        // Omitted arguments (e.g. =MMULT(,)) are not valid matrices.
        if matches!(args[0], Node::EmptyArgKind) || matches!(args[1], Node::EmptyArgKind) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "MMULT requires non-empty arrays".to_string(),
            );
        }

        let a = match self.eval_to_matrix(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let b = match self.eval_to_matrix(&args[1], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        match (a, b) {
            (Matrix::Numbers(a), Matrix::Numbers(b)) => multiply(a, b, cell),
            (a, b) => mmult_arrays(a.into_array(), b.into_array(), cell),
        }
    }

    /// Evaluates an argument of MMULT. It is what `eval_to_array` gives, with
    /// a range of numbers read straight into the numbers, and the transpose
    /// of one not built at all.
    fn eval_to_matrix(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Matrix, CalcResult> {
        if let Node::FunctionKind {
            kind: Function::Transpose,
            args,
        } = node
        {
            if args.len() == 1 {
                return Ok(self.eval_to_matrix(&args[0], cell)?.transpose());
            }
        }
        let result = self.evaluate_node_in_context(node, cell);
        if let CalcResult::Range { left, right } = result {
            if let Some(values) = self.evaluate_range_of_numbers(left, right) {
                return Ok(Matrix::Numbers(Numbers {
                    values,
                    rows: (right.row - left.row + 1) as usize,
                    columns: (right.column - left.column + 1) as usize,
                    transposed: false,
                }));
            }
        }
        Ok(Matrix::from_array(self.result_to_array(result, cell)?))
    }
}

/// MMULT of two arrays, whatever they hold. It is here for the arrays that are
/// not both matrices of numbers, which is to say for the error there is to
/// report: which one depends on the order of these checks, and of the values
/// that are not numbers the first one counts, row by row, in the first array
/// before the second.
fn mmult_arrays(
    a: Vec<Vec<ArrayNode>>,
    b: Vec<Vec<ArrayNode>>,
    cell: CellReferenceIndex,
) -> CalcResult {
    if a.is_empty() || a[0].is_empty() || b.is_empty() || b[0].is_empty() {
        return CalcResult::new_error(
            Error::VALUE,
            cell,
            "MMULT requires non-empty arrays".to_string(),
        );
    }

    let m = a.len();
    let k = a[0].len();
    let k2 = b.len();
    let n = b[0].len();

    if k != k2 {
        return CalcResult::new_error(Error::VALUE, cell, "MMULT dimension mismatch".to_string());
    }

    // Coerce one ArrayNode into f64 (or return an early error).
    // MMULT does not coerce booleans or empty cells — those yield #VALUE!.
    fn coerce(node: &ArrayNode, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        match node {
            ArrayNode::Number(v) => Ok(*v),
            ArrayNode::Boolean(_) | ArrayNode::Empty | ArrayNode::String(_) => {
                Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "MMULT requires numeric values".to_string(),
                ))
            }
            ArrayNode::Error(e) => Err(CalcResult::new_error(
                e.clone(),
                cell,
                "MMULT received an error value".to_string(),
            )),
        }
    }

    // Pre-coerce both matrices so we fail fast and avoid repeated work.
    let mut a_num = vec![vec![0.0f64; k]; m];
    for i in 0..m {
        if a[i].len() != k {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "MMULT array1 is not rectangular".to_string(),
            );
        }
        for j in 0..k {
            match coerce(&a[i][j], cell) {
                Ok(v) => a_num[i][j] = v,
                Err(e) => return e,
            }
        }
    }

    let mut b_num = vec![vec![0.0f64; n]; k];
    for i in 0..k {
        if b[i].len() != n {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "MMULT array2 is not rectangular".to_string(),
            );
        }
        for j in 0..n {
            match coerce(&b[i][j], cell) {
                Ok(v) => b_num[i][j] = v,
                Err(e) => return e,
            }
        }
    }

    let mut result: Vec<Vec<ArrayNode>> = Vec::with_capacity(m);
    // Classic matrix multiplication: C[i][j] = sum over p of A[i][p] * B[p][j].
    // Clippy can't see that i and j each address both matrices in the inner
    // loop, so named indices stay clearer than iterator combinators here.
    #[allow(clippy::needless_range_loop)]
    for i in 0..m {
        let mut row = Vec::with_capacity(n);
        for j in 0..n {
            let mut s = 0.0f64;
            for p in 0..k {
                s += a_num[i][p] * b_num[p][j];
            }
            if s.is_infinite() || s.is_nan() {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "MMULT result overflow".to_string(),
                );
            }
            row.push(ArrayNode::Number(s));
        }
        result.push(row);
    }

    CalcResult::Array(result)
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Numbers that do not add up exactly, so that the order of the sums shows.
    fn numbers(count: usize, seed: &mut u64) -> Vec<f64> {
        (0..count)
            .map(|_| {
                *seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((*seed >> 11) as f64 / (1u64 << 53) as f64 - 0.5) * 1e3
            })
            .collect()
    }

    /// The numbers of an array, bit for bit; `None` for anything else.
    fn bits(result: CalcResult) -> Option<Vec<Vec<u64>>> {
        let CalcResult::Array(array) = result else {
            return None;
        };
        array
            .iter()
            .map(|row| {
                row.iter()
                    .map(|value| match value {
                        ArrayNode::Number(number) => Some(number.to_bits()),
                        _ => None,
                    })
                    .collect()
            })
            .collect()
    }

    // The product of matrices of numbers is, to the last bit, what the
    // product of the same arrays is, transposed or not.
    #[test]
    fn numbers_multiply_like_arrays() {
        let cell = CellReferenceIndex {
            sheet: 0,
            row: 1,
            column: 1,
        };
        let mut seed = 7;
        for (m, k, n) in [
            (1, 1, 1),
            (2, 3, 4),
            (5, 1, 3),
            (1, 7, 1),
            (4, 6, 2),
            (3, 3, 3),
        ] {
            for a_transposed in [false, true] {
                for b_transposed in [false, true] {
                    // What is held is the transpose of the matrix, if transposed
                    let matrix = |rows: usize, columns: usize, transposed: bool, seed: &mut u64| {
                        let (rows, columns) = if transposed {
                            (columns, rows)
                        } else {
                            (rows, columns)
                        };
                        Numbers {
                            values: numbers(rows * columns, seed),
                            rows,
                            columns,
                            transposed,
                        }
                    };
                    let a = matrix(m, k, a_transposed, &mut seed);
                    let b = matrix(k, n, b_transposed, &mut seed);
                    let copy = |numbers: &Numbers| Numbers {
                        values: numbers.values.clone(),
                        ..*numbers
                    };
                    let expected = mmult_arrays(copy(&a).into_array(), copy(&b).into_array(), cell);
                    let product = multiply(a, b, cell);
                    assert_eq!(
                        bits(product).unwrap(),
                        bits(expected).unwrap(),
                        "{m}x{k} by {k}x{n}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_matrix_is_numbers_only_if_it_holds_nothing_else() {
        let number = ArrayNode::Number(1.0);
        let is_numbers = |array| matches!(Matrix::from_array(array), Matrix::Numbers(_));
        assert!(is_numbers(vec![vec![number.clone(); 3]; 2]));
        assert!(!is_numbers(vec![]));
        assert!(!is_numbers(vec![vec![]]));
        // not a rectangle
        assert!(!is_numbers(vec![
            vec![number.clone(); 2],
            vec![number.clone()]
        ]));
        for other in [
            ArrayNode::Empty,
            ArrayNode::Boolean(true),
            ArrayNode::String("1".to_string()),
            ArrayNode::Error(Error::DIV),
        ] {
            assert!(!is_numbers(vec![vec![number.clone(), other]]));
        }
        // transposing twice is doing nothing
        let array = vec![vec![ArrayNode::Number(1.0), ArrayNode::Number(2.0)]];
        let twice = Matrix::from_array(array.clone()).transpose().transpose();
        assert_eq!(twice.into_array(), array);
        let once = Matrix::from_array(array).transpose();
        assert_eq!(
            once.into_array(),
            vec![vec![ArrayNode::Number(1.0)], vec![ArrayNode::Number(2.0)]]
        );
    }
}
