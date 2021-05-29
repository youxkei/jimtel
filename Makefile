SOURCES := $(shell find . -name "*.rs")
PLUGINS := loudness_limiter loudness_ceiling

jimtel_linux.tar.gz:
	cargo build --release --workspace
	mkdir -p jimtel_linux
	@for plugin in ${PLUGINS}; do \
		mv target/release/lib$${plugin}.so jimtel_linux/jimtel_$${plugin}.so; \
	done
	tar cvzf jimtel_linux.tar.gz jimtel_linux
	rm -r jimtel_linux

jimtel_windows.zip:
	cargo build --release --workspace
	mkdir -p jimtel_windows
	@for plugin in ${PLUGINS}; do \
		mv target/release/$${plugin}.dll jimtel_windows/jimtel_$${plugin}.dll; \
	done
	powershell Compress-Archive jimtel_windows jimtel_windows.zip
	rm -r jimtel_windows

jimtel_macos.zip:
	cargo build --release --workspace
	mkdir -p jimtel_macos
	@for plugin in ${PLUGINS}; do \
		mv target/release/lib$${plugin}.dylib jimtel_macos/jimtel_$${plugin}.vst; \
	done
	zip -r jimtel_macos.zip jimtel_macos
	rm -r jimtel_macos
