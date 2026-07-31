#!/bin/bash
python -m venv venv
source venv/bin/activate
pip install patchelf
pip install maturin
pip install sphinx
pip install furo
maturin develop
cp ../../assets/logo/svg/orange+black.svg docs/_static/logo-light.svg
cp ../../assets/logo/svg/orange+white.svg docs/_static/logo-dark.svg
sphinx-build -M html docs html
python -m http.server --directory html/html/
