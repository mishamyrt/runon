.PHONY: help
help: ## print this message
	@awk \
		'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / \
		{printf "\033[33m%-15s\033[0m %s\n", $$1, $$2}' \
		$(MAKEFILE_LIST)

.PHONY: build
build: ## build runon
	swift build
	rm -rf ./dist
	mkdir ./dist
	cp .build/debug/runon ./dist/runon-daemon
	cp scripts/runon.bash ./dist/runon
	chmod +x ./dist/runon

.PHONY: lint
lint: ## check code style
	swiftlint lint --config .swiftlint.yaml .
	shellcheck -a scripts/runon.bash

install: ## install runon to the system
	rm -f \
		/usr/local/bin/runon \
		/usr/local/bin/runon-daemon
	cp dist/* /usr/local/bin/