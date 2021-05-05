SOURCES := $(shell find . -name "*.rs")

target/release/libjimtel_loudness_limiter.so: ${SOURCES}
	cargo build --release --workspace

.PHONY: install
install: target/release/libjimtel_loudness_limiter.so
	mkdir -p ~/.vst
	cp target/release/libjimtel_loudness_limiter.so ~/.vst
