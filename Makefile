SOURCES := $(shell find src -name "*.rs")

target/release/libjimtel.so: ${SOURCES}
	cargo build --release

.PHONY: install
install: target/release/libjimtel.so
	mkdir -p ~/.lv2/jimtel.lv2
	cp target/release/libjimtel.so ~/.lv2/jimtel.lv2/
	cp lv2/manifest.ttl            ~/.lv2/jimtel.lv2/
	cp lv2/jimtel.ttl              ~/.lv2/jimtel.lv2/
