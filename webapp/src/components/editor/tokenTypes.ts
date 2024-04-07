type ErrorType =
  | 'REF'
  | 'NAME'
  | 'VALUE'
  | 'DIV'
  | 'NA'
  | 'NUM'
  | 'ERROR'
  | 'NIMPL'
  | 'SPILL'
  | 'CALC'
  | 'CIRC';

type OpCompareType =
  | 'LessThan'
  | 'GreaterThan'
  | 'Equal'
  | 'LessOrEqualThan'
  | 'GreaterOrEqualThan'
  | 'NonEqual';

type OpSumType = 'Add' | 'Minus';

type OpProductType = 'Times' | 'Divide';

interface ReferenceType {
  sheet: string | null;
  row: number;
  column: number;
  absolute_column: boolean;
  absolute_row: boolean;
}

interface ParsedReferenceType {
  column: number;
  row: number;
  absolute_column: boolean;
  absolute_row: boolean;
}

interface Reference {
  Reference: ReferenceType;
}

interface Range {
  Range: {
    sheet: string | null;
    left: ParsedReferenceType;
    right: ParsedReferenceType;
  };
}

export type TokenType =
  | 'Illegal'
  | 'Eof'
  | { Ident: string }
  | { String: string }
  | { Boolean: boolean }
  | { Number: number }
  | { ERROR: ErrorType }
  | { COMPARE: OpCompareType }
  | { SUM: OpSumType }
  | { PRODUCT: OpProductType }
  | 'POWER'
  | 'LPAREN'
  | 'RPAREN'
  | 'COLON'
  | 'SEMICOLON'
  | 'LBRACKET'
  | 'RBRACKET'
  | 'LBRACE'
  | 'RBRACE'
  | 'COMMA'
  | 'BANG'
  | 'PERCENT'
  | 'AND'
  | Reference
  | Range;

export interface MarkedToken {
  token: TokenType;
  start: number;
  end: number;
}

export function tokenIsReferenceType(token: TokenType): token is Reference {
  return typeof token === 'object' && 'Reference' in token;
}

export function tokenIsRangeType(token: TokenType): token is Range {
  return typeof token === 'object' && 'Range' in token;
}
