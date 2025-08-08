# pip3 install maturin
default: node python
	cargo b --release

clean:
	rm -rf target
	rm -rf pkg
	rm -rf *.zip
	rm -rf timecode_js_node
	rm -rf timecode_js_web



python:
	python3 -m maturin build --features python --release
pydev: python
	python3 -m pip install ./target/wheels/timecodes-10.3.0-cp313-cp313-macosx_11_0_x86_64.whl --force-reinstall
	python3 test.py
node:
	wasm-pack build --release --target nodejs --features javascript
	mv pkg timecode_js_node
	zip -r timecode_js_node.zip timecode_js_node
web:
	wasm-pack build --release --target web --features javascript
	mv pkg timecode_js_web
	zip -r timecode_js_web.zip timecode_js_web
