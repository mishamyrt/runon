.PHONY: build
build:
	swift build
	rm -rf ./dist
	mkdir ./dist
	cp .build/debug/runif ./dist/

.PHONY: lint
lint:
	swiftlint lint --config .swiftlint.yaml .
