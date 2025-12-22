#!/bin/bash
set -euo pipefail

if command -v python3 &> /dev/null; then
    PYTHON=python3
else
    PYTHON=python
fi

$PYTHON -m venv venv
source venv/bin/activate
# Not sure why patchelf is needed
pip install patchelf maturin pytest
maturin develop
pytest tests/
