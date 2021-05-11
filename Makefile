ffi: include/bufro.h
	cargo build --release

include/bufro.h:
	cbindgen --config cbindgen.toml --crate bufro --output include/bufro.h

install:
	cp target/release/libbufro.so /usr/lib/libbufro.so
	cp include/* /usr/include
