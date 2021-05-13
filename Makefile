target/release/libbufro.so: include/bufro.h src/*.rs
	cargo build --release

include/bufro.h: src/*.rs cbindgen.toml
	cbindgen --config cbindgen.toml --crate bufro --output include/bufro.h

install:
	cp target/release/libbufro.so /usr/lib/libbufro.so
	cp include/* /usr/include

clean:
	rm -rf target
