# pip3 install maturin
default: node python
	cargo b

clean:
	rm -rf target
	rm -rf pkg


python:
	maturin build --features python
node:
	wasm-pack build --target nodejs --features javascript
web:
	wasm-pack build --target web --features javascript

release:
	mv pkg timecode_js
	zip -r timecode_js.zip timecode_js
