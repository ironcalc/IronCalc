#!/bin/bash
set -euo pipefail

# Define the directory containing the Python files
EXAMPLES_DIR="docs/examples"

# Check if the directory exists
if [ ! -d "$EXAMPLES_DIR" ]; then
  echo "Directory $EXAMPLES_DIR does not exist."
  exit 1
fi

if command -v python3 &> /dev/null; then
    PYTHON=python3
else
    PYTHON=python
fi

$PYTHON -m venv venv
source venv/bin/activate
pip install patchelf maturin pytest
maturin develop

# Iterate over all Python files in the examples directory
for file in "$EXAMPLES_DIR"/*.py; do
  # Check if there are any Python files
  if [ -f "$file" ]; then
    echo "Running $file..."
    if python "$file"; then
        echo "$file ran successfully"
    else
        echo "Error running $file"
        exit 1
    fi
  else
    echo "No Python files found in $EXAMPLES_DIR"
    exit 1
  fi
done
