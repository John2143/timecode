.PHONY: clean java

# pip3 install maturin
default: node python
	cargo b --release

clean:
	rm -rf target
	rm -rf pkg
	rm -rf java/*.class
	rm -rf java/libtimecode.*
	rm -rf java/*.h

python:
	python3 -m maturin build --features python --release

pydev: python
	python3 -m pip install ./target/wheels/timecode-0.1.0-cp310-cp310-macosx_10_7_x86_64.whl --force-reinstall
	python3 test.py

node:
	wasm-pack build --release --target nodejs --features javascript
	mv pkg timecode_js_node
	zip -r timecode_js_node.zip timecode_js_node

web:
	wasm-pack build --release --target web --features javascript
	mv pkg timecode_js_web
	zip -r timecode_js_web.zip timecode_js_web

java:
	cargo build --features jni
	cp ./target/debug/libtimecode.* java/
	rm -rf java/*.class
	javac java/test.java
	javac -h . java/test.java && mv *.h java
	cd java; java -cp . RTimecode
