VERSION := 1.0.2
ENTRYPOINT_SCRIPT_FILE := scripts/runon.bash
BUILD_INFO_FILE := Sources/BuildInfo.swift
BUILD_INFO_TEMPLATE := Sources/BuildInfo.template.swift

.PHONY: help
help: ## print this message
	@awk \
		'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / \
		{printf "\033[33m%-15s\033[0m %s\n", $$1, $$2}' \
		$(MAKEFILE_LIST)

.PHONY: generate
generate: ## generate build info
	@bash build/generate-info.sh \
		"${BUILD_INFO_TEMPLATE}" "${VERSION}" > "${BUILD_INFO_FILE}"

.PHONY: build
build: generate ## build runon
	swift build
	rm -rf ./dist
	mkdir ./dist
	cp .build/debug/runon ./dist/runon-daemon
	cp "${ENTRYPOINT_SCRIPT_FILE}" ./dist/runon
	chmod +x ./dist/runon

.PHONY: build-release
build-release: build
	bash ./build/generate-release-notes.sh "v${VERSION}" > dist/notes.md
	cd dist; zip -r runon.zip .

.PHONY: lint
lint: generate ## check code style
	swiftlint lint --config .swiftlint.yaml .
	shellcheck -a "${ENTRYPOINT_SCRIPT_FILE}"

.PHONY: install
install: ## install runon to the system
	rm -f \
		/usr/local/bin/runon \
		/usr/local/bin/runon-daemon
	cp dist/* /usr/local/bin/

.PHONY: test
test: generate ## run tests
	swift test

.PHONY: setup
setup: ## download dependencies
	swift package resolve

.PHONY: release
publish: # release a new version
	git tag "v${VERSION}"
	git chglog -o CHANGELOG.md
	git tag -d "v${VERSION}"
	git add Makefile
	git add CHANGELOG.md
	git commit -m "chore: release v${VERSION}"
	git tag "v${VERSION}"
	git push
	git push --tags
