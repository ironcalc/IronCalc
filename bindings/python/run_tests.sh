#!/bin/bash
set -euo pipefail

python3 -m venv venv
source venv/bin/activate
# Not sure why patchelf is needed
pip install patchelf maturin pytest
maturin develop
pytest tests/
