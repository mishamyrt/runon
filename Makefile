.PHONY: build
build:
	swift build
	rm -rf ./dist
	mkdir ./dist
	cp .build/debug/runif ./dist/runif-daemon
	cp scripts/runif.bash ./dist/runif
	chmod +x ./dist/runif

.PHONY: lint
lint:
	swiftlint lint --config .swiftlint.yaml .

install:
	cp dist/* /usr/local/bin/