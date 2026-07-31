#!/bin/bash
#
# Builds the static site deployed at https://ironcalc.dev into docs/ironcalc.dev/dist:
#
#   dist/index.html  landing page for developing / scripting IronCalc
#   dist/rust/       rust API docs (cargo doc) for the ironcalc and ironcalc_base crates
#   dist/python/     python bindings docs (sphinx)
#   dist/wasm/       wasm bindings docs (TODO)
#   dist/nodejs/     nodejs bindings docs (TODO)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIST_DIR="$SCRIPT_DIR/dist"

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Landing page
cp "$SCRIPT_DIR/index.html" "$DIST_DIR/index.html"
cp "$ROOT_DIR/assets/logo/svg/orange+black.svg" "$DIST_DIR/logo.svg"

# Rust documentation (https://ironcalc.dev/rust)
echo "Building rust docs..."
# remove docs of stale crates from previous builds
rm -rf "$ROOT_DIR/target/doc"
(cd "$ROOT_DIR" && cargo doc --no-deps --lib -p ironcalc -p ironcalc_base)
cp -r "$ROOT_DIR/target/doc" "$DIST_DIR/rust"
# cargo doc does not generate a top level index.html, redirect to the main crate
cat > "$DIST_DIR/rust/index.html" <<EOF
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="refresh" content="0; url=ironcalc/index.html" />
    <title>IronCalc rust documentation</title>
  </head>
  <body>
    <a href="ironcalc/index.html">Redirecting to the ironcalc crate documentation...</a>
  </body>
</html>
EOF

# Python bindings documentation (https://ironcalc.dev/python)
echo "Building python docs..."
cd "$ROOT_DIR/bindings/python"
if [ ! -d venv ]; then
  python3 -m venv venv
fi
source venv/bin/activate
pip install --quiet patchelf maturin sphinx furo
maturin develop
cp "$ROOT_DIR/assets/logo/svg/orange+black.svg" docs/_static/logo-light.svg
cp "$ROOT_DIR/assets/logo/svg/orange+white.svg" docs/_static/logo-dark.svg
rm -rf html
sphinx-build -M html docs html
deactivate
cp -r "$ROOT_DIR/bindings/python/html/html" "$DIST_DIR/python"

# Wasm bindings documentation (https://ironcalc.dev/wasm)
# TODO: coming soon page until the real documentation is written
mkdir -p "$DIST_DIR/wasm"
cp "$SCRIPT_DIR/wasm.html" "$DIST_DIR/wasm/index.html"

# Nodejs bindings documentation (https://ironcalc.dev/nodejs)
# TODO: coming soon page until the real documentation is written
mkdir -p "$DIST_DIR/nodejs"
cp "$SCRIPT_DIR/nodejs.html" "$DIST_DIR/nodejs/index.html"

echo "Site generated in $DIST_DIR"
echo "Preview it with: python3 -m http.server --directory $DIST_DIR"
