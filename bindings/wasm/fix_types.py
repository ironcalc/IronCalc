header = r"""
/* tslint:disable */
/* eslint-disable */
""".strip()

def fix_types(text: str):
    with open("types.ts") as f:
        types_str = f.read()
        header_types = "{}\n\n{}".format(header, types_str)
    
    text = text.replace(header, header_types)
    for line in text.splitlines():
        line = line.lstrip()
        # Skip internal methods
        if line.startswith("readonly model_"):
            continue
        if line.find("any") != -1:
            print("There are 'unfixed' public types. Please check.")
            exit(1)

    return text

if __name__ == "__main__":
    dts_files = ["pkg/ironcalc.d.ts", "pkg/xlsx.d.ts"]
    for types_file in dts_files:
        with open(types_file) as f:
            text = f.read()
        text = fix_types(text)
        with open(types_file, "wb") as f:
            f.write(bytes(text, "utf8"))

    js_files = ["pkg/ironcalc.js", "pkg/xlsx.js"]
    with open("types.js") as f:
        text_js = f.read()

    for js_file in js_files:
        with open(js_file) as f:
            text = f.read()
        with open(js_file, "wb") as f:
            f.write(bytes("{}\n{}".format(text_js, text), "utf8"))
