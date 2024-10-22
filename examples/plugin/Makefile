all: build_memory_wasm build_text cp_plugin

build_memory_wasm:
	cd memory_wasm && $(MAKE) build

build_text:
	cd text && $(MAKE) build

cp_plugin:
	mkdir -p ../../build/plugin
	cp ../../target/wasm32-wasip1/release/text.wasm ../../build/plugin/text.kpe