#!/bin/bash
python -m venv venv
source venv/bin/activate
pip install patchelf
pip install maturin
pip install sphinx
maturin develop
sphinx-build -M html docs html
python -m http.server --directory html/html/
