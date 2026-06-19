# Parity check between komadome (Ruby) and komadome-rs (Rust)
# See docs/parity.md for background.

DATE ?= $(shell date +%Y-%m-%d)
RUBY_KOMADOME ?= ../komadome
SHINONOME ?= ../shinonome
RUST_BUILD ?= build
RUBY_BUILD ?= $(RUBY_KOMADOME)/tmp/build
COMPARE ?= ruby scripts/compare_builds.rb
# ZIP_MODE=rust で komadome-rs の generate-zip を使用（デフォルト: shinonome）
ZIP_MODE ?= shinonome

.PHONY: parity parity-fixture parity-fast \
	setup-fixture generate-zips generate-zips-shinonome generate-zips-rust \
	build-rs build-ruby build-ruby-fixture \
	compare clean help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

# ---------------------------------------------------------------------------
# High-level targets
# ---------------------------------------------------------------------------

parity: generate-zips build-ruby build-rs compare ## Full parity check with dev DB (date-fixed builds + comparison)
	@echo "=== Parity check complete ==="

parity-fixture: setup-fixture generate-zips build-ruby build-rs compare ## Parity check with fixture data (fast, edge-case coverage)
	@echo "=== Parity check with fixture complete ==="

parity-fast: compare ## Compare existing build outputs (skip builds)
	@echo "=== Fast parity check complete (no builds) ==="

# ---------------------------------------------------------------------------
# Setup / generation
# ---------------------------------------------------------------------------

setup-fixture: ## Load fixture data into DB
	@echo "=== Loading fixture data ==="
	cd $(RUBY_KOMADOME) && PARITY_FIXTURE=1 bundle exec rails db:seed

generate-zips: ## Generate CSV zip files and copy to komadome
ifeq ($(ZIP_MODE),rust)
	$(MAKE) generate-zips-rust
else
	$(MAKE) generate-zips-shinonome
endif

generate-zips-shinonome: ## Generate CSV zip files via shinonome rake task (into komadome)
	@echo "=== Generating ZIP files (shinonome -> komadome) ==="
	cd $(SHINONOME) && CSV_DIR=$(RUBY_KOMADOME)/data/csv_zip bundle exec rails csv:create

generate-zips-rust: ## Generate CSV zip files via komadome-rs (into komadome-rs)
	@echo "=== Generating ZIP files (komadome-rs) ==="
	cargo run --release -- generate-zip

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

build-rs: ## Build Rust output with fixed date
	@echo "=== Building Rust (DATE=$(DATE)) ==="
	cargo run --release -- --date $(DATE) export
	cargo run --release -- build

build-ruby: ## Build Ruby output with fixed date
	@echo "=== Building Ruby (DATE=$(DATE)) ==="
	cd $(RUBY_KOMADOME) && KOMADOME_BUILD_DATE=$(DATE) bundle exec rake build:all

build-ruby-fixture: setup-fixture generate-zips build-ruby ## Build Ruby output with fixture data
	@echo "=== Building Ruby with fixture (DATE=$(DATE)) ==="

# ---------------------------------------------------------------------------
# Compare / clean
# ---------------------------------------------------------------------------

compare: ## Run two-mode comparison
	@echo "=== Comparing builds ==="
	$(COMPARE) $(RUBY_BUILD) $(RUST_BUILD)

clean: ## Remove Rust build output
	rm -rf $(RUST_BUILD)/*
