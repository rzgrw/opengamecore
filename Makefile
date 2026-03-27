.PHONY: build run test clippy fmt clean install bundle

build:
	cargo build --workspace

release:
	cargo build --release --workspace

run:
	cargo run -p opengamecore-app

cli:
	cargo run -p opengamecore-cli -- $(ARGS)

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

check: fmt clippy test
	@echo "All checks passed!"

clean:
	cargo clean

install: release
	cp target/release/opengamecore-app /usr/local/bin/opengamecore
	cp target/release/ogc /usr/local/bin/ogc
	@echo "Installed opengamecore and ogc to /usr/local/bin"

uninstall:
	rm -f /usr/local/bin/opengamecore /usr/local/bin/ogc
	@echo "Uninstalled opengamecore and ogc"

VERSION ?= 0.1.0
bundle: release
	./scripts/bundle-macos.sh $(VERSION)
