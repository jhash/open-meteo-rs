.PHONY: clippy
clippy:
	cargo clippy --all-targets -- -D warnings

.PHONY: test
test:
	cargo test

.PHONY: fmt
fmt:
	cargo fmt
	dprint fmt

.PHONY: fmt-check
fmt-check:
	cargo fmt --check
	dprint check

.PHONY: deny
deny:
	cargo deny check
