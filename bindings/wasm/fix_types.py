# Regrettably at the time of writing there is not a perfect way to
# generate the TypeScript types from Rust so we basically fix them manually
# Hopefully this will suffice for our needs and one day will be automatic

header = r"""
/* tslint:disable */
/* eslint-disable */
""".strip()

get_tokens_str = r"""
* @returns {any}
*/
export function getTokens(formula: string): any;
""".strip()

get_tokens_str_types = r"""
* @returns {TokenType[]}
*/
export function getTokens(formula: string): TokenType[];
""".strip()

update_style_str = r"""
/**
* @param {any} range
* @param {string} style_path
* @param {string} value
*/
  updateRangeStyle(range: any, style_path: string, value: string): void;
""".strip()

update_style_str_types = r"""
/**
* @param {Area} range
* @param {string} style_path
* @param {string} value
*/
  updateRangeStyle(range: Area, style_path: string, value: string): void;
""".strip()

properties = r"""
/**
* @returns {any}
*/
  getWorksheetsProperties(): any;
""".strip()

properties_types = r"""
/**
* @returns {WorksheetProperties[]}
*/
  getWorksheetsProperties(): WorksheetProperties[];
""".strip()

style = r"""
* @returns {any}
*/
  getCellStyle(sheet: number, row: number, column: number): any;
""".strip()

style_types = r"""
* @returns {CellStyle}
*/
  getCellStyle(sheet: number, row: number, column: number): CellStyle;
""".strip()

def fix_types(text):
    text = text.replace(get_tokens_str, get_tokens_str_types)
    text = text.replace(update_style_str, update_style_str_types)
    text = text.replace(properties, properties_types)
    text = text.replace(style, style_types)
    with open("types.ts") as f:
        types_str = f.read()
        header_types = "{}\n\n{}".format(header, types_str)
    text = text.replace(header, header_types)
    return text
    


if __name__ == "__main__":
    types_file = "pkg/wasm.d.ts"
    with open(types_file) as f:
        text = f.read()
    text = fix_types(text)
    with open(types_file, "wb") as f:
        f.write(bytes(text, "utf8"))

    js_file = "pkg/wasm.js"
    with open("types.js") as f:
        text_js = f.read()
    with open(js_file) as f:
        text = f.read()

    with open(js_file, "wb") as f:
        f.write(bytes("{}\n{}".format(text_js, text), "utf8"))
    

    
