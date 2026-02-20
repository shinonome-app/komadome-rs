#!/bin/bash
# Compare build outputs between Ruby and Rust versions
# Usage: ./scripts/compare_builds.sh <ruby_build_dir> <rust_build_dir>

set -euo pipefail

RUBY_DIR="${1:?Usage: $0 <ruby_build_dir> <rust_build_dir>}"
RUST_DIR="${2:?Usage: $0 <ruby_build_dir> <rust_build_dir>}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
EXTRACT_SCRIPT="$SCRIPT_DIR/extract_text.rb"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "=== Step 1: File list comparison ==="

# Get sorted list of HTML files (relative paths)
(cd "$RUBY_DIR" && find . -name '*.html' | sort) > "$TMPDIR/ruby_files.txt"
(cd "$RUST_DIR" && find . -name '*.html' | sort) > "$TMPDIR/rust_files.txt"

RUBY_ONLY=$(comm -23 "$TMPDIR/ruby_files.txt" "$TMPDIR/rust_files.txt")
RUST_ONLY=$(comm -13 "$TMPDIR/ruby_files.txt" "$TMPDIR/rust_files.txt")
COMMON=$(comm -12 "$TMPDIR/ruby_files.txt" "$TMPDIR/rust_files.txt")

RUBY_ONLY_COUNT=$(echo "$RUBY_ONLY" | grep -c . || true)
RUST_ONLY_COUNT=$(echo "$RUST_ONLY" | grep -c . || true)
COMMON_COUNT=$(echo "$COMMON" | grep -c . || true)

echo "  Common files: $COMMON_COUNT"
echo "  Ruby only:    $RUBY_ONLY_COUNT"
echo "  Rust only:    $RUST_ONLY_COUNT"

if [[ $RUBY_ONLY_COUNT -gt 0 ]]; then
  echo ""
  echo "  Files only in Ruby:"
  echo "$RUBY_ONLY" | head -20 | sed 's/^/    /'
  if [[ $RUBY_ONLY_COUNT -gt 20 ]]; then
    echo "    ... ($((RUBY_ONLY_COUNT - 20)) more)"
  fi
fi

if [[ $RUST_ONLY_COUNT -gt 0 ]]; then
  echo ""
  echo "  Files only in Rust:"
  echo "$RUST_ONLY" | head -20 | sed 's/^/    /'
  if [[ $RUST_ONLY_COUNT -gt 20 ]]; then
    echo "    ... ($((RUST_ONLY_COUNT - 20)) more)"
  fi
fi

echo ""
echo "=== Step 2: Content comparison (text extraction) ==="

if [[ ! -f "$EXTRACT_SCRIPT" ]]; then
  echo "Warning: extract_text.rb not found at $EXTRACT_SCRIPT"
  echo "Skipping content comparison."
  exit 0
fi

DIFF_COUNT=0
SAME_COUNT=0
SAMPLE_LIMIT=500

i=0
while IFS= read -r file; do
  ((i++))
  if [[ $i -gt $SAMPLE_LIMIT ]]; then
    echo "  (Sampled $SAMPLE_LIMIT of $COMMON_COUNT common files)"
    break
  fi

  ruby_text=$(ruby "$EXTRACT_SCRIPT" "$RUBY_DIR/$file" 2>/dev/null || echo "EXTRACT_ERROR")
  rust_text=$(ruby "$EXTRACT_SCRIPT" "$RUST_DIR/$file" 2>/dev/null || echo "EXTRACT_ERROR")

  if [[ "$ruby_text" == "$rust_text" ]]; then
    ((SAME_COUNT++))
  else
    ((DIFF_COUNT++))
    if [[ $DIFF_COUNT -le 10 ]]; then
      echo "  DIFF: $file"
      diff <(echo "$ruby_text") <(echo "$rust_text") | head -10 | sed 's/^/    /'
    fi
  fi
done <<< "$COMMON"

echo ""
echo "==============================="
echo "Content comparison: $SAME_COUNT same, $DIFF_COUNT different"
echo "File list: $COMMON_COUNT common, $RUBY_ONLY_COUNT Ruby-only, $RUST_ONLY_COUNT Rust-only"
echo "==============================="
