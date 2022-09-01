# pip3 install maturin
default: node python
	cargo b

clean:
	rm -rf target
	rm -rf pkg


python:
	maturin build --features python --release
node:
	wasm-pack build --release --target nodejs --features javascript
	mv pkg timecode_js_node
	zip -r timecode_js_node.zip timecode_js_node
web:
	wasm-pack build --release --target web --features javascript
	mv pkg timecode_js_web
	zip -r timecode_js_web.zip timecode_js_web
