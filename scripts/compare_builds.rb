#!/usr/bin/env ruby
# frozen_string_literal: true

# Compare build outputs between Ruby (komadome) and Rust (komadome-rs) versions.
#
# Two-mode comparison (see docs/parity.md):
#   BYTE       : raw bytes are identical                    -> Phase 2 達成
#   NORMALIZED : differ in bytes but canonical DOM は一致   -> Phase 1 達成 / Phase 2 未達
#   DIFF       : canonical DOM すら不一致                    -> ロジック差・要修正
#
# Phase 1 のゲートは DIFF=0、Phase 2 の進捗は種別ごとの BYTE 一致率で測る。
#
# Usage: ruby scripts/compare_builds.rb <ruby_build_dir> <rust_build_dir>

begin
  require "nokogiri"
rescue LoadError
  $stderr.puts "ERROR: nokogiri gem is required. Install with: gem install nokogiri"
  exit 1
end
require "set"

# --- normalization ----------------------------------------------------------

# Canonicalize HTML so that engine-level cosmetic differences (whitespace,
# attribute order, self-closing style, newline noise) collapse away while
# tag structure / attributes / text content are preserved.
#
# Two files with the same canonical form are considered NORMALIZED-equal.
def canonicalize(html)
  doc = Nokogiri::HTML5(html)
  canonicalize_node(doc)
  doc.to_html
end

def canonicalize_node(node)
  # Drop HTML comments: they do not affect rendering. Notably komadome (Ruby)
  # emits `<!-- generated at: ... -->` on card/top pages that komadome-rs omits;
  # at the NORMALIZED level that is noise, not a logic difference. Such pages
  # therefore land in NORMALIZED (byte-different, semantically equal), which is
  # exactly what Phase 2 tracks.
  if node.comment?
    node.remove
    return
  end

  if node.element?
    # sort attributes so attribute order does not matter
    attrs = node.attribute_nodes.sort_by(&:name)
    attrs.each { |a| node.remove_attribute(a.name) }
    attrs.each { |a| node[a.name] = a.value }
  end

  if node.text?
    # Absorb indent / newline noise: collapse runs to a single space, strip ends.
    # Whitespace-only text nodes (block-element indentation) become empty and are
    # dropped. This can swallow a significant inline space in rare cases; that gap
    # is intentionally a Phase 1 tolerance and will resurface in the Phase 2 byte
    # comparison.
    collapsed = node.content.gsub(/\s+/, " ").strip
    if collapsed.empty?
      node.remove
    else
      node.content = collapsed
    end
  else
    node.children.to_a.each { |child| canonicalize_node(child) }
  end
end

# Cache canonical form per absolute path to avoid re-parsing in the example pass.
CANON_CACHE = {}
def canonical_of(path)
  CANON_CACHE[path] ||= canonicalize(File.read(path, encoding: "utf-8"))
end

# --- classification ---------------------------------------------------------

# Returns :byte, :normalized, or :diff for a common file.
def classify(ruby_path, rust_path)
  ruby_bytes = File.read(ruby_path, encoding: "utf-8")
  rust_bytes = File.read(rust_path, encoding: "utf-8")
  return :byte if ruby_bytes == rust_bytes

  return :normalized if canonicalize(ruby_bytes) == canonicalize(rust_bytes)

  :diff
end

# Group key for per-type reporting (cards/, index_pages/sakuhin_*, person{id}, ...).
def category_of(file)
  base = File.basename(file, ".html")
  return base unless file.include?("/")

  top = file.split("/").first
  sub =
    if base.match?(/^list_inp/) then "list_inp*"
    elsif base.match?(/^person_inp_/) then "person_inp_*"
    elsif base.match?(/^person_all_/) then "person_all_*"
    elsif base.match?(/^person\d/) then "person{id}"
    elsif base.match?(/^sakuhin_inp_/) then "sakuhin_inp_*"
    elsif base.match?(/^sakuhin_/) then "sakuhin_*"
    elsif base.match?(/^work_inp/) then "work_inp*"
    elsif base.match?(/^work\d/) then "work{page}"
    elsif base.match?(/^whatsnew/) then "whatsnew*"
    elsif base.match?(/^soramoyou/) then "soramoyou*"
    elsif top == "cards" then "show"
    else base
    end
  "#{top}/#{sub}"
end

def find_html_files(dir)
  Dir.glob("**/*.html", base: dir).sort
end

def simple_diff(a_text, b_text, max_lines: 10)
  a_lines = a_text.lines
  b_lines = b_text.lines
  output = []
  shown = 0
  [a_lines.size, b_lines.size].max.times do |i|
    al = a_lines[i]
    bl = b_lines[i]
    next if al == bl

    output << "    - #{al.chomp}" if al
    output << "    + #{bl.chomp}" if bl
    shown += 1
    break if shown >= max_lines
  end
  output.join("\n")
end

# --- main -------------------------------------------------------------------

ruby_dir = ARGV[0]
rust_dir = ARGV[1]

unless ruby_dir && rust_dir
  $stderr.puts "Usage: #{$0} <ruby_build_dir> <rust_build_dir>"
  exit 1
end

unless Dir.exist?(ruby_dir) && Dir.exist?(rust_dir)
  $stderr.puts "ERROR: Both directories must exist"
  $stderr.puts "  ruby_dir: #{ruby_dir} (#{Dir.exist?(ruby_dir) ? 'exists' : 'NOT FOUND'})"
  $stderr.puts "  rust_dir: #{rust_dir} (#{Dir.exist?(rust_dir) ? 'exists' : 'NOT FOUND'})"
  exit 1
end

puts "=== Step 1: File list comparison ==="

ruby_files = find_html_files(ruby_dir)
rust_files = find_html_files(rust_dir)
rust_set = rust_files.to_set
ruby_set = ruby_files.to_set

common = ruby_files.select { |f| rust_set.include?(f) }
ruby_only = ruby_files.reject { |f| rust_set.include?(f) }
rust_only = rust_files.reject { |f| ruby_set.include?(f) }

puts "  Common files: #{common.size}"
puts "  Ruby only:    #{ruby_only.size}"
puts "  Rust only:    #{rust_only.size}"

if ruby_only.any?
  puts ""
  puts "  Files only in Ruby:"
  ruby_only.first(20).each { |f| puts "    #{f}" }
  puts "    ... (#{ruby_only.size - 20} more)" if ruby_only.size > 20
end

if rust_only.any?
  puts ""
  puts "  Files only in Rust:"
  rust_only.first(20).each { |f| puts "    #{f}" }
  puts "    ... (#{rust_only.size - 20} more)" if rust_only.size > 20
end

puts ""
puts "=== Step 2: Two-mode content comparison ==="

# state[file] => :byte | :normalized | :diff ; nil on error
states = {}
errors = []
max_examples = 10

common.each_with_index do |file, i|
  if i > 0 && (i % 1000).zero?
    $stderr.print "\r  Progress: #{i}/#{common.size}..."
  end
  states[file] = classify(File.join(ruby_dir, file), File.join(rust_dir, file))
rescue => e
  states[file] = nil
  errors << "#{file}: #{e.message}" if errors.size < max_examples
end
$stderr.print "\r#{' ' * 40}\r" if common.size >= 1000

byte       = states.select { |_, s| s == :byte }.keys
normalized = states.select { |_, s| s == :normalized }.keys
diff       = states.select { |_, s| s == :diff }.keys
error      = states.select { |_, s| s.nil? }.keys

# DIFF examples first (most actionable: real logic divergence)
diff_examples = diff.reject { |f| f.start_with?("cards/") }
diff_examples = diff if diff_examples.empty?
if diff_examples.any?
  puts ""
  puts "  DIFF examples (canonical DOM differs; showing first #{[diff_examples.size, 5].min}):"
  diff_examples.first(5).each do |file|
    puts "  DIFF: #{file}"
    puts simple_diff(canonical_of(File.join(ruby_dir, file)),
                     canonical_of(File.join(rust_dir, file)))
    puts ""
  end
end

# --- per-category table -----------------------------------------------------

puts ""
puts "=== Step 3: Per-category breakdown (byte / normalized / diff) ==="

by_cat = Hash.new { |h, k| h[k] = { byte: 0, normalized: 0, diff: 0, error: 0 } }
states.each do |file, s|
  key = s.nil? ? :error : s
  by_cat[category_of(file)][key] += 1
end

name_w = by_cat.keys.map(&:size).max || 10
printf("  %-#{name_w}s  %7s %7s %7s %7s   %s\n",
       "category", "byte", "norm", "diff", "error", "byte%")
by_cat.sort_by { |cat, _| cat }.each do |cat, c|
  total = c[:byte] + c[:normalized] + c[:diff] + c[:error]
  byte_pct = total.zero? ? 0 : (100.0 * c[:byte] / total)
  printf("  %-#{name_w}s  %7d %7d %7d %7d   %5.1f%%\n",
         cat, c[:byte], c[:normalized], c[:diff], c[:error], byte_pct)
end

# --- summary ----------------------------------------------------------------

total = common.size
byte_pct = total.zero? ? 0 : (100.0 * byte.size / total)
puts ""
puts "==============================="
puts "BYTE       : #{byte.size}"
puts "NORMALIZED : #{normalized.size}   (Phase 1 OK / Phase 2 残)"
puts "DIFF       : #{diff.size}   (要修正)"
puts "ERROR      : #{error.size}"
puts "-------------------------------"
puts "File list  : #{common.size} common, #{ruby_only.size} Ruby-only, #{rust_only.size} Rust-only"
printf("Byte match : %.1f%% (%d/%d)\n", byte_pct, byte.size, total)
puts "Phase 1 gate (DIFF==0 && file lists match): " \
     "#{diff.empty? && ruby_only.empty? && rust_only.empty? ? 'PASS' : 'FAIL'}"
puts "Phase 2 goal (BYTE==100%): " \
     "#{byte.size == total && ruby_only.empty? && rust_only.empty? ? 'PASS' : "#{byte_pct.round(1)}%"}"

if errors.any?
  puts "-------------------------------"
  puts "Errors:"
  errors.each { |e| puts "  #{e}" }
end
puts "==============================="

# Exit non-zero when the Phase 1 gate fails, so CI can use this directly.
gate_pass = diff.empty? && ruby_only.empty? && rust_only.empty? && error.empty?
exit(gate_pass ? 0 : 1)
