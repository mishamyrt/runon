.PHONY: build build-release check lint test install measure

build:
	cargo build --locked -p runon

lint:
	cargo fmt --all --check
	cargo clippy --locked --workspace --all-targets -- -D warnings
	for script in scripts/install.sh build/package.sh build/generate-release-notes.sh; do bash -n "$$script"; done

test:
	cargo test --locked --workspace

check: lint test

build-release:
	bash build/package.sh

install:
	cargo build --release --locked -p runon
	mkdir -p "$(HOME)/.local/bin"
	install -m 755 target/release/runon "$(HOME)/.local/bin/.runon-new"
	mv -f "$(HOME)/.local/bin/.runon-new" "$(HOME)/.local/bin/runon"

measure:
	cargo build --release --locked -p runon
	python3 scripts/measure.py --service
	cargo run --release --locked -p runon-runtime --example benchmark
