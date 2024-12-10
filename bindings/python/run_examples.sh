#!/bin/bash

# Define the directory containing the Python files
EXAMPLES_DIR="docs/examples"

# Check if the directory exists
if [ ! -d "$EXAMPLES_DIR" ]; then
  echo "Directory $EXAMPLES_DIR does not exist."
  exit 1
fi

python -m venv venv
source venv/bin/activate
# not sure why this is needed
pip install patchelf
pip install maturin
pip install pytest
maturin develop

# Iterate over all Python files in the examples directory
for file in "$EXAMPLES_DIR"/*.py; do
  # Check if there are any Python files
  if [ -f "$file" ]; then
    echo "Running $file..."
    python "$file"
    
    # Check if the script ran successfully
    if [ $? -ne 0 ]; then
      echo "Error running $file"
    else
      echo "$file ran successfully"
    fi
  else
    echo "No Python files found in $EXAMPLES_DIR"
    exit 1
  fi
done
