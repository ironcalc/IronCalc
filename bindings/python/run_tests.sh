#!/bin/bash
python -m venv venv
source venv/bin/activate
# not sure why this is needed
pip install patchelf
pip install maturin
pip install pytest
maturin develop
pytest tests/
