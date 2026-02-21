#!/usr/bin/env ruby
# frozen_string_literal: true

# Compare build outputs between Ruby and Rust versions
# Usage: ruby scripts/compare_builds.rb <ruby_build_dir> <rust_build_dir>

begin
  require "nokogiri"
rescue LoadError
  $stderr.puts "ERROR: nokogiri gem is required. Install with: gem install nokogiri"
  exit 1
end

def extract_text(path)
  html = File.read(path, encoding: "utf-8")
  doc = Nokogiri::HTML(html)
  doc.css("script, style").each(&:remove)
  doc.text
    .lines
    .map { |l| l.gsub(/[ \t]+/, " ").strip }
    .reject(&:empty?)
    .join("\n")
end

def find_html_files(dir)
  Dir.glob("**/*.html", base: dir).sort
end

def simple_diff(a_text, b_text, max_lines: 10)
  a_lines = a_text.lines
  b_lines = b_text.lines
  output = []
  max_i = [a_lines.size, b_lines.size].max

  shown = 0
  max_i.times do |i|
    al = a_lines[i]
    bl = b_lines[i]
    next if al == bl

    if al && bl
      output << "    - #{al.chomp}"
      output << "    + #{bl.chomp}"
    elsif al
      output << "    - #{al.chomp}"
    else
      output << "    + #{bl.chomp}"
    end
    shown += 1
    break if shown >= max_lines
  end
  output.join("\n")
end

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

ruby_set = ruby_files.to_set
rust_set = rust_files.to_set

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
puts "=== Step 2: Content comparison (text extraction) ==="

same_count = 0
card_diff_count = 0
non_card_diff_count = 0
error_count = 0
errors = []
card_diffs = []
non_card_diffs = []
max_examples = 10
all_non_card_diffs = []

common.each_with_index do |file, i|
  if i > 0 && (i % 1000).zero?
    $stderr.print "\r  Progress: #{i}/#{common.size}..."
  end

  ruby_text = extract_text(File.join(ruby_dir, file))
  rust_text = extract_text(File.join(rust_dir, file))

  if ruby_text == rust_text
    same_count += 1
  elsif file.start_with?("cards/")
    card_diff_count += 1
    card_diffs << file if card_diffs.size < max_examples
  else
    non_card_diff_count += 1
    all_non_card_diffs << file
    non_card_diffs << file if non_card_diffs.size < max_examples
  end
rescue => e
  error_count += 1
  errors << "#{file}: #{e.message}" if errors.size < max_examples
end

$stderr.print "\r" if common.size >= 1000

diff_count = card_diff_count + non_card_diff_count

# Show non-card diffs first (more actionable)
if non_card_diffs.any?
  puts "  Non-card diffs (#{non_card_diff_count} total, showing first #{non_card_diffs.size}):"
  non_card_diffs.each do |file|
    ruby_text = extract_text(File.join(ruby_dir, file))
    rust_text = extract_text(File.join(rust_dir, file))
    puts "  DIFF: #{file}"
    puts simple_diff(ruby_text, rust_text)
    puts ""
  end
end

if card_diffs.any?
  puts "  Card diffs (#{card_diff_count} total, showing first #{card_diffs.size}):"
  card_diffs.first(5).each do |file|
    ruby_text = extract_text(File.join(ruby_dir, file))
    rust_text = extract_text(File.join(rust_dir, file))
    puts "  DIFF: #{file}"
    puts simple_diff(ruby_text, rust_text)
    puts ""
  end
end

puts ""
puts "==============================="
puts "Content comparison: #{same_count} same, #{diff_count} different (#{card_diff_count} cards, #{non_card_diff_count} non-cards), #{error_count} errors"
puts "File list: #{common.size} common, #{ruby_only.size} Ruby-only, #{rust_only.size} Rust-only"

# Categorize non-card diffs by directory/prefix
if all_non_card_diffs.any?
  categories = all_non_card_diffs.group_by { |f| f.split("/").first }
  puts ""
  puts "Non-card diff breakdown:"
  categories.sort_by { |_, v| -v.size }.each do |cat, files|
    subcats = files.group_by { |f|
      basename = File.basename(f, ".html")
      if basename.match?(/^list_inp/)
        "list_inp*"
      elsif basename.match?(/^person_inp_/)
        "person_inp_*"
      elsif basename.match?(/^person_all_/)
        "person_all_*"
      elsif basename.match?(/^person\d/)
        "person{id}"
      elsif basename.match?(/^work_inp/)
        "work_inp*"
      elsif basename.match?(/^work\d/)
        "work{page}"
      elsif basename.match?(/^whatsnew/)
        "whatsnew*"
      elsif basename.match?(/^soramoyou/)
        "soramoyou*"
      else
        basename
      end
    }
    subcats.sort_by { |_, v| -v.size }.each do |subcat, subfiles|
      puts "  #{cat}/#{subcat}: #{subfiles.size}"
    end
  end
end
if errors.any?
  puts "Errors:"
  errors.each { |e| puts "  #{e}" }
end
puts "==============================="
