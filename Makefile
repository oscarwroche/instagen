## build: build rust binary
.PHONY: build
build:
	cargo build

## clippy: run clippy checks
.PHONY: clippy
clippy:
	cargo clippy

## run: build & launch server with tracing
.PHONY: run
run:
	RUST_LOG=tower_http=trace cargo run -- --key-file ssl/key.pem --cert-file ssl/cert.pem

## run: build & launch server with tracing + debug logs
.PHONY: run-debug
run-debug:
	RUST_LOG=debug,tower_http=trace cargo run -- --key-file ssl/key.pem --cert-file ssl/cert.pem

## css: build tailwindcss
.PHONY: css
css:
	tailwindcss -i static/css/input.css -o static/css/output.css --minify

## css: build tailwindcss, watch mode
.PHONY: css-watch
css-watch:
	tailwindcss -i static/css/input.css -o static/css/output.css --watch
