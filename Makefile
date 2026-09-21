.PHONY: test test-fixed-point check

test:
	cargo test --lib

test-fixed-point:
	cargo test --lib fixed_point_protocol

check:
	cargo check --lib
