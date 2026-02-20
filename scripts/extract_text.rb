#!/usr/bin/env ruby
# Extract text content from HTML files, stripping tags and normalizing whitespace.
# Used for semantic comparison between Ruby and Rust generated HTML.
#
# Usage: ruby extract_text.rb <html_file>

require "nokogiri"

file = ARGV[0]
unless file
  $stderr.puts "Usage: #{$0} <html_file>"
  exit 1
end

html = File.read(file, encoding: "UTF-8")
doc = Nokogiri::HTML(html)

# Remove script and style elements
doc.css("script, style").each(&:remove)

# Extract text, normalize whitespace
text = doc.text
  .gsub(/[ \t]+/, " ")      # collapse horizontal whitespace
  .gsub(/\n{3,}/, "\n\n")   # collapse excessive newlines
  .strip

puts text
