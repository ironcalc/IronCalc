header = r"""
/* tslint:disable */
/* eslint-disable */
""".strip()

def fix_types(text: str):
    with open("types.ts") as f:
        types_str = f.read()
        header_types = "{}\n\n{}".format(header, types_str)
    
    text = text.replace(header, header_types)
    inside_init_output = False
    for line in text.splitlines():
        stripped = line.lstrip()
        # Skip the entire InitOutput interface — it contains raw WASM function
        # pointer signatures whose JsValue params/returns legitimately use `any`.
        if stripped.startswith("export interface InitOutput {"):
            inside_init_output = True
        if inside_init_output:
            if stripped == "}":
                inside_init_output = False
            continue
        if stripped.find("any") != -1:
            print("There are 'unfixed' public types. Please check.")
            exit(1)

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
    