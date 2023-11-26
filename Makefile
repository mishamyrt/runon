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
	rm -f \
		/usr/local/bin/runif \
		/usr/local/bin/runif-daemon
	cp dist/* /usr/local/bin/