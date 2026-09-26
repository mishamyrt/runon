VERSION = 0.1.0
TARGET = target/release/runon
PREFIX ?= $(HOME)/.local/bin

all: $(TARGET)

$(TARGET): Cargo.toml Cargo.lock $(shell find crates -type f)
	cargo build --release --locked -p runon

.PHONY: lint
lint:
	cargo fmt --all --check
	cargo clippy --locked --workspace --all-targets -- -D warnings

.PHONY: test
test:
	cargo test --locked --workspace

check: lint test

install: $(TARGET)
	@mkdir -p "$(PREFIX)"
	install -m 755 "$(TARGET)" "$(PREFIX)/.runon-new"
	mv -f "$(PREFIX)/.runon-new" "$(PREFIX)/runon"

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
