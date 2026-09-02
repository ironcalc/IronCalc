use std::fmt;
use std::sync::OnceLock;

use bitcode::__private::{Buffer, Decoder, Encoder, Result as BitcodeResult, View};
use bitcode::{Decode, Encode};
use std::num::NonZeroUsize;

use crate::collab::fractional_key::FractionalKey;
use crate::collab::patch::{DefinedNameId, SheetId};
use crate::expressions::parser::ArrayNode;
use crate::expressions::token::{self, OpCompare, OpProduct, OpSum, OpUnary};
use crate::functions::Function;

/// Caps the work any decoded payload can ask of a consumer that walks the stream.
pub const MAX_TOKENS: usize = 8192;

/// Which sheet a reference points at.
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum StableSheetRef {
    /// Ref without explicit sheet prefix: resolves to the host cell's sheet.
    Current,
    Sheet(SheetId),
}

/// One axis (row or column) of a reference.
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct StableAxisRef {
    pub key: FractionalKey,
    /// Not used for addressing (the key is the whole address); preserves `$` display and copy/fill
    /// rebind semantics.
    pub absolute: bool,
}

/// One cell of an inline array literal.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum ArrayValue {
    Boolean(bool),
    Number(f64),
    String(String),
    Error(token::Error),
    Empty,
}

impl From<&ArrayNode> for ArrayValue {
    fn from(value: &ArrayNode) -> Self {
        match value {
            ArrayNode::Boolean(v) => ArrayValue::Boolean(*v),
            ArrayNode::Number(v) => ArrayValue::Number(*v),
            ArrayNode::String(v) => ArrayValue::String(v.clone()),
            ArrayNode::Error(v) => ArrayValue::Error(v.clone()),
            ArrayNode::Empty => ArrayValue::Empty,
        }
    }
}

impl From<&ArrayValue> for ArrayNode {
    fn from(value: &ArrayValue) -> Self {
        match value {
            ArrayValue::Boolean(v) => ArrayNode::Boolean(*v),
            ArrayValue::Number(v) => ArrayNode::Number(*v),
            ArrayValue::String(v) => ArrayNode::String(v.clone()),
            ArrayValue::Error(v) => ArrayNode::Error(v.clone()),
            ArrayValue::Empty => ArrayNode::Empty,
        }
    }
}

/// A `LAMBDA` parameter declaration.
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct LambdaParam {
    pub name: String,
    pub optional: bool,
}

/// A single step of the post-order stream: pops its arity off the operand stack and pushes one value.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum StableToken {
    // arity 0
    Boolean(bool),
    Number(f64),
    String(String),
    CellRef {
        sheet: StableSheetRef,
        row: StableAxisRef,
        column: StableAxisRef,
    },
    RangeRef {
        sheet: StableSheetRef,
        row1: StableAxisRef,
        column1: StableAxisRef,
        row2: StableAxisRef,
        column2: StableAxisRef,
    },
    DefinedName(DefinedNameId),
    TableName(String),
    /// Lambda/LET-bound variable, resolved lexically at eval; carries no stable id.
    NamedVariable(String),
    Error(token::Error),
    EmptyArg,
    Array(Vec<Vec<ArrayValue>>),
    /// Unparseable formula stored verbatim; must be the ONLY token in the stream.
    RawText(String),

    // arity 1
    Unary(OpUnary),
    ImplicitIntersection {
        automatic: bool,
    },
    SpillRange,

    // arity 2
    OpRange,
    OpConcatenate,
    OpPower,
    OpSum(OpSum),
    OpProduct(OpProduct),
    Compare(OpCompare),

    // variadic
    Function {
        kind: Function,
        argc: u16,
    },
    /// Non-builtin call (lambda param or defined-name lambda), by authored name.
    NamedFunction {
        name: String,
        argc: u16,
    },
    /// Pops 1 (the body). Parameters are payload, not stack operands.
    LambdaDef {
        parameters: Vec<LambdaParam>,
    },
    /// Pops argc args plus the lambda value beneath them (lambda pushed first, then args).
    LambdaCall {
        argc: u16,
    },
}

impl StableToken {
    /// How many operands this token pops before pushing its own result.
    fn arity(&self) -> usize {
        match self {
            StableToken::Boolean(_)
            | StableToken::Number(_)
            | StableToken::String(_)
            | StableToken::CellRef { .. }
            | StableToken::RangeRef { .. }
            | StableToken::DefinedName(_)
            | StableToken::TableName(_)
            | StableToken::NamedVariable(_)
            | StableToken::Error(_)
            | StableToken::EmptyArg
            | StableToken::Array(_)
            | StableToken::RawText(_) => 0,
            StableToken::Unary(_)
            | StableToken::ImplicitIntersection { .. }
            | StableToken::SpillRange
            | StableToken::LambdaDef { .. } => 1,
            StableToken::OpRange
            | StableToken::OpConcatenate
            | StableToken::OpPower
            | StableToken::OpSum(_)
            | StableToken::OpProduct(_)
            | StableToken::Compare(_) => 2,
            StableToken::Function { argc, .. } | StableToken::NamedFunction { argc, .. } => {
                *argc as usize
            }
            StableToken::LambdaCall { argc } => *argc as usize + 1,
        }
    }
}

/// Why a token stream is not a well-formed formula.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaError {
    /// A formula is at least one token.
    Empty,
    /// A token at `at` asked for more operands than the stack held.
    StackUnderflow { at: usize },
    /// The stream left `count` values on the stack instead of one.
    LeftoverOperands { count: usize },
    /// [`StableToken::RawText`] appeared alongside other tokens.
    MisplacedRawText,
    /// More than [`MAX_TOKENS`] tokens.
    TooManyTokens { count: usize },
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormulaError::Empty => write!(f, "formula has no tokens"),
            FormulaError::StackUnderflow { at } => {
                write!(
                    f,
                    "token at index {at} pops more operands than are available"
                )
            }
            FormulaError::LeftoverOperands { count } => {
                write!(f, "{count} operands left on the stack, expected 1")
            }
            FormulaError::MisplacedRawText => {
                write!(f, "raw text must be the only token in the stream")
            }
            FormulaError::TooManyTokens { count } => {
                write!(f, "{count} tokens exceeds the {MAX_TOKENS} token limit")
            }
        }
    }
}

impl std::error::Error for FormulaError {}

/// A validated post-order token stream.
///
/// The derived `Decode` builds one without running [`StableFormula::validate`], so anything read
/// back from untrusted bytes must be re-validated before it is walked.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct StableFormula(Vec<StableToken>);

impl StableFormula {
    pub fn new(tokens: Vec<StableToken>) -> Result<Self, FormulaError> {
        Self::validate(&tokens)?;
        Ok(StableFormula(tokens))
    }

    pub fn tokens(&self) -> &[StableToken] {
        &self.0
    }

    /// Points every reference naming `source` at `target` instead, returning whether anything
    /// changed. A reference with no sheet prefix stays as it is. Swapping a sheet ref is
    /// one-for-one, so a valid stream stays one.
    pub(crate) fn retarget_sheet(&mut self, source: SheetId, target: SheetId) -> bool {
        let mut changed = false;
        for token in &mut self.0 {
            let sheet = match token {
                StableToken::CellRef { sheet, .. } | StableToken::RangeRef { sheet, .. } => sheet,
                _ => continue,
            };
            if *sheet == StableSheetRef::Sheet(source) {
                *sheet = StableSheetRef::Sheet(target);
                changed = true;
            }
        }
        changed
    }

    /// Single stack-simulation pass. Consumers rebuilding a tree from a validated stream must do so
    /// iteratively too: the nesting depth comes from the payload, so recursion is a stack overflow.
    pub fn validate(tokens: &[StableToken]) -> Result<(), FormulaError> {
        if tokens.is_empty() {
            return Err(FormulaError::Empty);
        }
        if tokens.len() > MAX_TOKENS {
            return Err(FormulaError::TooManyTokens {
                count: tokens.len(),
            });
        }
        let mut depth = 0usize;
        for (at, token) in tokens.iter().enumerate() {
            if matches!(token, StableToken::RawText(_)) && tokens.len() != 1 {
                return Err(FormulaError::MisplacedRawText);
            }
            let arity = token.arity();
            if depth < arity {
                return Err(FormulaError::StackUnderflow { at });
            }
            depth = depth - arity + 1;
        }
        if depth != 1 {
            return Err(FormulaError::LeftoverOperands { count: depth });
        }
        Ok(())
    }
}

/// [`Function`] has more than the 256 variants bitcode's derive supports, so the coder pair is
/// written by hand over the variant index. Encoding is a plain fieldless-enum cast; decoding maps
/// back through a table built from [`Function::into_iter`], and rejects codes with no variant.
fn function_table() -> &'static [Option<Function>] {
    static TABLE: OnceLock<Vec<Option<Function>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let all: Vec<Function> = Function::into_iter().collect();
        let mut table = vec![None; all.len()];
        for f in all {
            let code = f.clone() as usize;
            if code < table.len() {
                table[code] = Some(f);
            }
        }
        table
    })
}

fn function_from_code(code: u16) -> Option<Function> {
    function_table().get(code as usize).cloned().flatten()
}

#[derive(Default)]
pub struct FunctionEncoder(<u16 as Encode>::Encoder);

impl Buffer for FunctionEncoder {
    fn collect_into(&mut self, out: &mut Vec<u8>) {
        self.0.collect_into(out);
    }

    fn reserve(&mut self, additional: NonZeroUsize) {
        self.0.reserve(additional);
    }
}

impl Encoder<Function> for FunctionEncoder {
    #[inline]
    fn encode(&mut self, t: &Function) {
        self.0.encode(&(t.clone() as u16));
    }
}

impl Encode for Function {
    type Encoder = FunctionEncoder;
}

#[derive(Default)]
pub struct FunctionDecoder<'a>(<u16 as Decode<'a>>::Decoder);

impl<'a> View<'a> for FunctionDecoder<'a> {
    fn populate(&mut self, input: &mut &'a [u8], length: usize) -> BitcodeResult<()> {
        // bitcode allows validation only here, so the codes are read twice: once to check them,
        // once for real.
        let mut probe = <u16 as Decode<'a>>::Decoder::default();
        probe.populate(&mut { *input }, length)?;
        for _ in 0..length {
            if function_from_code(probe.decode()).is_none() {
                return bitcode::__private::invalid_enum_variant();
            }
        }
        self.0.populate(input, length)
    }
}

impl<'a> Decoder<'a, Function> for FunctionDecoder<'a> {
    #[inline]
    fn decode(&mut self) -> Function {
        function_from_code(self.0.decode()).expect("code checked in populate")
    }
}

impl<'a> Decode<'a> for Function {
    type Decoder = FunctionDecoder<'a>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collab::fractional_index::virtual_key;

    fn axis(ordinal: u32, absolute: bool) -> StableAxisRef {
        StableAxisRef {
            key: virtual_key(ordinal),
            absolute,
        }
    }

    fn cell(sheet: StableSheetRef, row: u32, column: u32, absolute: bool) -> StableToken {
        StableToken::CellRef {
            sheet,
            row: axis(row, absolute),
            column: axis(column, absolute),
        }
    }

    /// Every `StableToken` and `StableSheetRef` variant, spread over representative formulas.
    #[test]
    fn round_trip() {
        let streams = vec![
            // =A1+$B$2
            vec![
                cell(StableSheetRef::Current, 1, 1, false),
                cell(StableSheetRef::Current, 2, 2, true),
                StableToken::OpSum(OpSum::Add),
            ],
            // =SUM(Sheet2!A1:B3)
            vec![
                StableToken::RangeRef {
                    sheet: StableSheetRef::Sheet(7),
                    row1: axis(1, false),
                    column1: axis(1, false),
                    row2: axis(3, true),
                    column2: axis(2, true),
                },
                StableToken::Function {
                    kind: Function::Sum,
                    argc: 1,
                },
            ],
            // =Sheet2!A1&"x"
            vec![
                cell(StableSheetRef::Sheet(7), 1, 1, false),
                StableToken::String("x".to_string()),
                StableToken::OpConcatenate,
            ],
            // =IF(A1>0,"y",-A1%)
            vec![
                cell(StableSheetRef::Current, 1, 1, false),
                StableToken::Number(0.0),
                StableToken::Compare(OpCompare::GreaterThan),
                StableToken::String("y".to_string()),
                cell(StableSheetRef::Current, 1, 1, false),
                StableToken::Unary(OpUnary::Percentage),
                StableToken::Unary(OpUnary::Minus),
                StableToken::Function {
                    kind: Function::If,
                    argc: 3,
                },
            ],
            // =MyName*Table1^2/3
            vec![
                StableToken::DefinedName(42),
                StableToken::TableName("Table1".to_string()),
                StableToken::Number(2.0),
                StableToken::OpPower,
                StableToken::OpProduct(OpProduct::Times),
                StableToken::Number(3.0),
                StableToken::OpProduct(OpProduct::Divide),
            ],
            // =LAMBDA(a,[b],a*b)(2,3)
            vec![
                StableToken::NamedVariable("a".to_string()),
                StableToken::NamedVariable("b".to_string()),
                StableToken::OpProduct(OpProduct::Times),
                StableToken::LambdaDef {
                    parameters: vec![
                        LambdaParam {
                            name: "a".to_string(),
                            optional: false,
                        },
                        LambdaParam {
                            name: "b".to_string(),
                            optional: true,
                        },
                    ],
                },
                StableToken::Number(2.0),
                StableToken::Number(3.0),
                StableToken::LambdaCall { argc: 2 },
            ],
            // ={TRUE,1;"s",#REF!;,} with a custom call over it
            vec![
                StableToken::Array(vec![
                    vec![ArrayValue::Boolean(true), ArrayValue::Number(1.0)],
                    vec![
                        ArrayValue::String("s".to_string()),
                        ArrayValue::Error(token::Error::REF),
                    ],
                    vec![ArrayValue::Empty, ArrayValue::Empty],
                ]),
                StableToken::EmptyArg,
                StableToken::NamedFunction {
                    name: "myLambda".to_string(),
                    argc: 2,
                },
            ],
            // =@A1:B2# with an error leaf joined by the range operator
            vec![
                cell(StableSheetRef::Current, 1, 1, false),
                StableToken::Error(token::Error::VALUE),
                StableToken::OpRange,
                StableToken::SpillRange,
                StableToken::ImplicitIntersection { automatic: true },
                StableToken::Boolean(false),
                StableToken::ImplicitIntersection { automatic: false },
                StableToken::OpConcatenate,
            ],
            vec![StableToken::RawText("=this is not a formula".to_string())],
        ];

        for tokens in streams {
            let formula = StableFormula::new(tokens.clone()).expect("valid stream");
            let bytes = bitcode::encode(&formula);
            let decoded: StableFormula = bitcode::decode(&bytes).expect("round trip");
            assert_eq!(decoded, formula);
            assert_eq!(decoded.tokens(), tokens.as_slice());
            assert_eq!(StableFormula::validate(decoded.tokens()), Ok(()));
        }

        // The hand-written `Function` coder is only total if the table has no holes.
        for kind in Function::into_iter() {
            let tokens = vec![StableToken::Function { kind, argc: 0 }];
            let formula = StableFormula::new(tokens.clone()).unwrap();
            let decoded: StableFormula = bitcode::decode(&bitcode::encode(&formula)).unwrap();
            assert_eq!(decoded.tokens(), tokens.as_slice());
        }
    }

    #[test]
    fn validator_rejects() {
        let one = || StableToken::Boolean(true);
        let cases: Vec<(Vec<StableToken>, FormulaError)> = vec![
            (vec![], FormulaError::Empty),
            (
                vec![one(), StableToken::OpConcatenate],
                FormulaError::StackUnderflow { at: 1 },
            ),
            (
                vec![one(), one()],
                FormulaError::LeftoverOperands { count: 2 },
            ),
            (
                vec![StableToken::RawText("x".to_string()), one()],
                FormulaError::MisplacedRawText,
            ),
            (
                vec![
                    one(),
                    one(),
                    StableToken::Function {
                        kind: Function::Sum,
                        argc: 3,
                    },
                ],
                FormulaError::StackUnderflow { at: 2 },
            ),
            (
                vec![one(), StableToken::LambdaCall { argc: 1 }],
                FormulaError::StackUnderflow { at: 1 },
            ),
            (
                vec![one(); MAX_TOKENS + 1],
                FormulaError::TooManyTokens {
                    count: MAX_TOKENS + 1,
                },
            ),
        ];

        for (tokens, expected) in cases {
            assert_eq!(StableFormula::new(tokens).unwrap_err(), expected);
        }
    }

    #[test]
    fn adversarial_decode() {
        let formula = StableFormula::new(vec![
            cell(StableSheetRef::Sheet(1), 1, 1, false),
            StableToken::Function {
                kind: Function::Sum,
                argc: 1,
            },
        ])
        .unwrap();
        let bytes = bitcode::encode(&formula);

        let mut corruptions: Vec<Vec<u8>> = (0..bytes.len()).map(|n| bytes[..n].to_vec()).collect();
        for i in 0..bytes.len() {
            for flip in [0x01u8, 0x80, 0xff] {
                let mut c = bytes.clone();
                c[i] ^= flip;
                corruptions.push(c);
            }
        }
        corruptions.push(vec![0xff; 64]);

        for bytes in corruptions {
            // Whatever comes back must be re-validatable without panicking.
            if let Ok(decoded) = bitcode::decode::<StableFormula>(&bytes) {
                let _ = StableFormula::validate(decoded.tokens());
            }
        }

        // Decoded-but-invalid: the derive skips `new`, so validation has to happen on the way out.
        let unvalidated =
            StableFormula(vec![StableToken::Boolean(true), StableToken::Boolean(true)]);
        let decoded: StableFormula = bitcode::decode(&bitcode::encode(&unvalidated)).unwrap();
        assert_eq!(
            StableFormula::validate(decoded.tokens()),
            Err(FormulaError::LeftoverOperands { count: 2 })
        );
    }
}
