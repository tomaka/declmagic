#
#  Note: this makefile is used IN ADDITION TO Cargo.toml, it is NOT an alternative to Cargo
#

all: nphysics gl stb-image



nphysics: deps/nphysics/lib/nphysics.timestamp

deps/nphysics/lib/nphysics.timestamp: 
	cd deps/nphysics && $(MAKE) deps && cd ../..
	cd deps/nphysics && $(MAKE) && cd ../..
	cp deps/nphysics/lib/* target/deps
	cp deps/nphysics/ncollide/lib/* target/deps
	cp deps/nphysics/ncollide/nalgebra/lib/* target/deps
	touch deps/nphysics/lib/nphysics.timestamp



gl: target/deps/gl.timestamp

target/deps/gl.timestamp: 
	rustc deps/gl/gl.rs --out-dir target/deps
	touch target/deps/gl.timestamp




stb-image: target/deps/stb-image.timestamp

target/deps/stb-image.timestamp: 
	mkdir -p deps/stb-image/cmake-build
	gcc -o tmp.o -c deps/stb-image/stb_image.c
	ar crf target/deps/libstb-image.a tmp.o
	rm tmp.o
	rustc --out-dir target/deps -L target/deps deps/stb-image/lib.rs
	touch target/deps/stb-image.timestamp



clean:
	cd deps/nphysics && $(MAKE) clean
	rm -rf target
