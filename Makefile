VERSION = 0.1.0

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

.PHONY: publish
publish:
	@sed -E 's/^version = "[^"]+"/version = "${VERSION}"/' Cargo.toml > Cargo.toml.tmp
	@mv Cargo.toml.tmp Cargo.toml
	@cargo update -p runon
	@git add Makefile Cargo.toml Cargo.lock
	@git commit -m "chore: release ${VERSION} 🔥"
	@git tag "v${VERSION}"
	@git-cliff -o CHANGELOG.md
	@git tag -d "v${VERSION}"
	@git add CHANGELOG.md
	@git commit --amend --no-edit
	@git tag -a "v${VERSION}" -m "release v${VERSION}"
	@git push
	@git push --tags
