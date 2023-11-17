.PHONY: build
build:
	swift build
	rm -rf ./dist
	mkdir ./dist
	cp .build/arm64-apple-macosx/debug/runif ./dist/