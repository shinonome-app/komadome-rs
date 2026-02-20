#!/bin/bash
# Compare JSONL exports between Ruby and Rust versions
# Usage: ./scripts/compare_exports.sh <ruby_export_dir> <rust_export_dir>

set -euo pipefail

RUBY_DIR="${1:?Usage: $0 <ruby_export_dir> <rust_export_dir>}"
RUST_DIR="${2:?Usage: $0 <ruby_export_dir> <rust_export_dir>}"

if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed." >&2
  exit 1
fi

FILES=(
  masters.json
  cards.jsonl
  person_pages.jsonl
  work_indexes.jsonl
  person_indexes.jsonl
  whatsnew.jsonl
  news.jsonl
  top.json
  wip_work_indexes.jsonl
  wip_person_indexes.jsonl
  person_all_indexes.jsonl
  list_inp.jsonl
)

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

PASS=0
FAIL=0
SKIP=0

for file in "${FILES[@]}"; do
  ruby_file="$RUBY_DIR/$file"
  rust_file="$RUST_DIR/$file"

  if [[ ! -f "$ruby_file" ]]; then
    echo "SKIP $file (Ruby file not found)"
    ((SKIP++))
    continue
  fi
  if [[ ! -f "$rust_file" ]]; then
    echo "SKIP $file (Rust file not found)"
    ((SKIP++))
    continue
  fi

  # Normalize JSON: sort keys, one object per line
  if [[ "$file" == *.jsonl ]]; then
    # For JSONL: sort keys per line, then sort lines
    jq -S -c '.' "$ruby_file" | sort > "$TMPDIR/ruby_$file"
    jq -S -c '.' "$rust_file" | sort > "$TMPDIR/rust_$file"
  else
    # For JSON: sort keys
    jq -S '.' "$ruby_file" > "$TMPDIR/ruby_$file"
    jq -S '.' "$rust_file" > "$TMPDIR/rust_$file"
  fi

  if diff -q "$TMPDIR/ruby_$file" "$TMPDIR/rust_$file" > /dev/null 2>&1; then
    echo "PASS $file"
    ((PASS++))
  else
    echo "FAIL $file"
    diff --unified=3 "$TMPDIR/ruby_$file" "$TMPDIR/rust_$file" | head -50
    echo "  ..."
    ((FAIL++))
  fi
done

echo ""
echo "==============================="
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped (of ${#FILES[@]} files)"
echo "==============================="

exit $FAIL
