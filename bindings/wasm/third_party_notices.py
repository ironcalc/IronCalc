# Generates a THIRD-PARTY-NOTICES.md for a binary built from a Rust crate.
#
# It lists every crate that gets compiled into the artifact (as resolved by
# `cargo tree` for the given target and features, proc-macros excluded since
# they only run at build time) together with the license files found in each
# crate's package, grouping crates that ship byte-identical license texts.
#
# It only needs `cargo` and the crate sources already in the cargo registry
# (they are after any build), so it does not require cargo-about or
# cargo-license to be installed.
#
# Usage:
#   third_party_notices.py --manifest-path Cargo.toml --name "@ironcalc/wasm" \
#       --output pkg/THIRD-PARTY-NOTICES.md \
#       [--target wasm32-unknown-unknown] [--features xlsx] \
#       [--include ../../base/THIRD-PARTY-NOTICES.md]...
#
# `--include` files are copied verbatim before the crate list; use them for
# notices that do not come from a crate (bundled data, standard library...).
import argparse
import json
import os
import re
import subprocess
import sys

LICENSE_FILE_RE = re.compile(
    r"^(LICEN[CS]E|COPYING|NOTICE|UNLICENSE)([-_.].*)?$", re.IGNORECASE
)
# Standard library and runtime pieces that are linked into every Rust binary
# but are not crates in Cargo.lock.
RUST_STD_NOTICE = """\
## Rust standard library

The compiled code also contains the Rust standard library (`core`, `alloc`,
`std`) and the support crates it links (such as `compiler_builtins` and the
`dlmalloc` allocator on WebAssembly). They are distributed under the MIT or
Apache-2.0 licenses, at your option; `compiler_builtins` additionally
contains portions of LLVM's compiler-rt under Apache-2.0 WITH LLVM-exception.
The full notices are in the Rust repository:
https://github.com/rust-lang/rust/blob/master/COPYRIGHT
"""


def run(cmd, cwd):
    return subprocess.run(
        cmd, cwd=cwd, check=True, capture_output=True, text=True
    ).stdout


def linked_packages(manifest_dir, target, features):
    """(name, version) of every crate compiled into the artifact."""
    cmd = [
        "cargo",
        "tree",
        "--edges",
        "normal,no-proc-macro",
        "--prefix",
        "none",
        "--format",
        "{p}",
    ]
    if target:
        cmd += ["--target", target]
    if features:
        cmd += ["--features", ",".join(features)]
    packages = set()
    for line in run(cmd, manifest_dir).splitlines():
        line = line.strip()
        if not line or line.endswith("(*)"):
            continue
        # "name vX.Y.Z" or "name vX.Y.Z (/path or url)"
        name, version = line.split(" ")[:2]
        packages.add((name, version.lstrip("v")))
    return packages


def package_metadata(manifest_dir, features):
    """(name, version) -> package record from `cargo metadata`."""
    cmd = ["cargo", "metadata", "--format-version", "1"]
    if features:
        cmd += ["--features", ",".join(features)]
    out = run(cmd, manifest_dir)
    return {(p["name"], p["version"]): p for p in json.loads(out)["packages"]}


def license_files(package):
    """Sorted list of (file name, text) for the license files in a crate."""
    crate_dir = os.path.dirname(package["manifest_path"])
    names = set()
    if package.get("license_file"):
        names.add(package["license_file"])
    for entry in os.listdir(crate_dir):
        if LICENSE_FILE_RE.match(entry) and os.path.isfile(
            os.path.join(crate_dir, entry)
        ):
            names.add(entry)
    files = []
    for name in sorted(names):
        path = os.path.join(crate_dir, name)
        if not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8", errors="replace") as f:
            files.append((os.path.basename(name), f.read()))
    return files


def normalize(text):
    return "\n".join(line.rstrip() for line in text.strip().splitlines())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest-path", required=True)
    parser.add_argument("--name", required=True, help="artifact name for the title")
    parser.add_argument("--output", required=True)
    parser.add_argument("--target")
    parser.add_argument("--features", action="append", default=[])
    parser.add_argument("--include", action="append", default=[])
    args = parser.parse_args()

    manifest_dir = os.path.dirname(os.path.abspath(args.manifest_path))
    metadata = package_metadata(manifest_dir, args.features)
    # Crates from the workspace (source == None) are IronCalc itself.
    third_party = sorted(
        key
        for key in linked_packages(manifest_dir, args.target, args.features)
        if metadata[key].get("source")
    )

    out = []
    out.append("# Third-party notices for {}\n".format(args.name))
    out.append(
        "This file lists the third-party components distributed with {} and "
        "their license notices. IronCalc itself is licensed under MIT or "
        "Apache-2.0, at your option (see LICENSE-MIT and LICENSE-Apache-2.0).\n".format(
            args.name
        )
    )
    out.append(
        "Generated by `bindings/wasm/third_party_notices.py` from Cargo.lock. "
        "Do not edit by hand.\n"
    )

    for path in args.include:
        with open(path, encoding="utf-8") as f:
            text = f.read().strip()
        # Demote the included file's headings one level under this document.
        text = re.sub(r"^(#+)", r"#\1", text, flags=re.MULTILINE)
        out.append(text + "\n")

    out.append(RUST_STD_NOTICE)

    out.append("## Rust crates\n")
    out.append("| Crate | Version | License | Repository |")
    out.append("|-------|---------|---------|------------|")
    texts = {}  # normalized text -> (file name, text, [crate labels])
    missing = []
    for key in third_party:
        package = metadata[key]
        label = "{} {}".format(*key)
        out.append(
            "| {} | {} | {} | {} |".format(
                key[0],
                key[1],
                package.get("license") or "see repository",
                package.get("repository") or "",
            )
        )
        files = license_files(package)
        if not files:
            missing.append((label, package))
        for file_name, text in files:
            entry = texts.setdefault(normalize(text), (file_name, text, []))
            entry[2].append(label)
    out.append("")

    if missing:
        out.append("### Crates without license files in their package\n")
        out.append(
            "The following crates declare a license but do not ship the license "
            "text in the crate package; the text is available from their "
            "repositories.\n"
        )
        for label, package in missing:
            out.append(
                "- {}: {} ({})".format(
                    label, package.get("license"), package.get("repository") or ""
                )
            )
        out.append("")

    out.append("## License texts\n")
    for _, (file_name, text, labels) in sorted(
        texts.items(), key=lambda item: (item[1][2][0], item[1][0])
    ):
        out.append("### {} ({})\n".format(file_name, ", ".join(labels)))
        out.append("```text")
        out.append(text.strip())
        out.append("```\n")

    with open(args.output, "w", encoding="utf-8") as f:
        f.write("\n".join(out))
    print(
        "{}: {} crates, {} license texts, {} crates without license files".format(
            args.output, len(third_party), len(texts), len(missing)
        ),
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
