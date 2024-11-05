build:
	cargo build --bin run

clippy:
	cargo clippy -p kernel --all-features -- -D warnings
	cargo +nighlty clippy -p kernel --all-features -- -D warnings

run:
	cargo run --bin run -- --test --uefi

scp: all
	scp -i $(KEY) -P $(PORT) -o "StrictHostKeyChecking no" uefi.img $(DEST)	
	
clean:
	rm -f uefi.img

distclean: clean
	cargo clean

.DEFAULT_GOAL: build
.PHONY: run clean distclean
