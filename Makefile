target/release/libjimtel.so: src/lib.rs
	cargo build --release

.PHONY: install
install: target/release/libjimtel.so
	mkdir -p ~/.lv2/jimtel.lv2
	cp target/release/libjimtel.so ~/.lv2/jimtel.lv2/
	cp lv2/manifest.ttl            ~/.lv2/jimtel.lv2/
	cp lv2/jimtel.ttl              ~/.lv2/jimtel.lv2/
