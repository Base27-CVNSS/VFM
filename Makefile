.PHONY: test fixture verify

test:
	cargo test --all-targets

fixture:
	python tools/python_reference.py --fixtures-root .

verify:
	python tools/verify_independent.py
	cargo run --quiet --bin vfm -- build-minimal /tmp/rust-core-minimal.vfm
	cmp /tmp/rust-core-minimal.vfm fixtures/golden/core-minimal.vfm
