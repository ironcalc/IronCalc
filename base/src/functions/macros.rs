#[macro_export]
macro_rules! single_number_fn {
    // The macro takes:
    //   1) A function name to define (e.g. fn_sin)
    //   2) The operation to apply (e.g. f64::sin)
    ($fn_name:ident, $op:expr) => {
        pub(crate) fn $fn_name(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
            // 1) Check exactly one argument
            if args.len() != 1 {
                return CalcResult::new_args_number_error(cell);
            }
            // 2) Try to get a "NumberOrArray"
            match self.get_number_or_array(&args[0], cell) {
                // -----------------------------------------
                // Case A: It's a single number
                // -----------------------------------------
                Ok(NumberOrArray::Number(f)) => {
                    let x = $op(f);
                    if x.is_infinite() {
                        CalcResult::Error {
                            error: Error::DIV,
                            origin: cell,
                            message: "Divide by 0".to_string(),
                        }
                    } else if x.is_nan() {
                        CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Invalid number".to_string(),
                        }
                    } else {
                        CalcResult::Number(x)
                    }
                }

                // -----------------------------------------
                // Case B: It's an array, so apply $op
                // element-by-element.
                // -----------------------------------------
                Ok(NumberOrArray::Array(a)) => {
                    let mut array = Vec::new();
                    for row in a {
                        let mut data_row = Vec::with_capacity(row.len());
                        for value in row {
                            match value {
                                // If Boolean, treat as 0.0 or 1.0
                                ArrayNode::Boolean(b) => {
                                    let n = if b { 1.0 } else { 0.0 };
                                    let x = $op(n);
                                    if x.is_infinite() {
                                        data_row.push(ArrayNode::Error(Error::DIV));
                                    } else if x.is_nan() {
                                        data_row.push(ArrayNode::Error(Error::VALUE));
                                    } else {
                                        data_row.push(ArrayNode::Number(x));
                                    }
                                }
                                // If Number, apply directly
                                ArrayNode::Number(n) => {
                                    let x = $op(n);
                                    if x.is_infinite() {
                                        data_row.push(ArrayNode::Error(Error::DIV));
                                    } else if x.is_nan() {
                                        data_row.push(ArrayNode::Error(Error::VALUE));
                                    } else {
                                        data_row.push(ArrayNode::Number(x));
                                    }
                                }
                                // If String, parse to f64 then apply or #VALUE! error
                                ArrayNode::String(s) => {
                                    let node = match s.parse::<f64>() {
                                        Ok(f) => {
                                            let x = $op(f);
                                            if x.is_infinite() {
                                                ArrayNode::Error(Error::DIV)
                                            } else if x.is_nan() {
                                                ArrayNode::Error(Error::VALUE)
                                            } else {
                                                ArrayNode::Number(x)
                                            }
                                        }
                                        Err(_) => ArrayNode::Error(Error::VALUE),
                                    };
                                    data_row.push(node);
                                }
                                // If Error, propagate the error
                                e @ ArrayNode::Error(_) => {
                                    data_row.push(e);
                                }
                            }
                        }
                        array.push(data_row);
                    }
                    CalcResult::Array(array)
                }

                // -----------------------------------------
                // Case C: It's an Error => just return it
                // -----------------------------------------
                Err(err_result) => err_result,
            }
        }
    };
}
