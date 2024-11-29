#!/bin/bash
set -euo pipefail

# Get the list of .pol files in the examples directory
pol_files=$(find examples -maxdepth 1 -name "*.pol" -printf "%f\n" | sort)

# Extract the 'path' entries from index.json
index_paths=$(jq -r '.[].path' examples/index.json | sort)

# Compare the two lists
if diff <(echo "$pol_files") <(echo "$index_paths") >/dev/null; then
  echo "Success: index.json and .pol files are in sync."
else
  echo "Error: index.json and .pol files in examples/ are not in sync."
  echo "Diff:"
  diff <(echo "$pol_files") <(echo "$index_paths")
fi
