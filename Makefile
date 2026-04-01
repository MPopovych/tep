.PHONY: build test lint fmt check install install-dev uninstall dist clean publish-check help

# Default
all: build

build:          ## Build debug binary
	cargo build

release:        ## Build optimized release binary
	cargo build --release
	@echo "Binary at: target/release/tep"

test:           ## Run all tests
	cargo test

lint:           ## Run clippy lints
	cargo clippy -- -D warnings

fmt:            ## Check formatting
	cargo fmt --check

check: lint fmt test  ## Full CI check: lint + fmt + test

install:        ## Install tep to ~/.cargo/bin
	cargo install --path .

install-dev:    ## Install debug build to ~/.cargo/bin (faster)
	cargo install --path . --debug

uninstall:      ## Remove installed binary
	cargo uninstall tep

dist:           ## Build release binary and copy to ./bin/
	cargo build --release
	mkdir -p bin
	cp target/release/tep bin/tep
	@echo "Dist binary: bin/tep"

clean:          ## Clean build artifacts
	cargo clean

publish-check:  ## Dry-run crate publish check
	cargo publish --dry-run --allow-dirty

help:           ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*## ' $(MAKEFILE_LIST) | \
	    awk 'BEGIN {FS = ":.*## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
