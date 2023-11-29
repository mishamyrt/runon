.PHONY: build
build:
	swift build
	rm -rf ./dist
	mkdir ./dist
	cp .build/debug/runon ./dist/runon-daemon
	cp scripts/runon.bash ./dist/runon
	chmod +x ./dist/runon

.PHONY: lint
lint:
	swiftlint lint --config .swiftlint.yaml .

install:
	rm -f \
		/usr/local/bin/runon \
		/usr/local/bin/runon-daemon
	cp dist/* /usr/local/bin/