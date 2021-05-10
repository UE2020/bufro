ffi:
		cargo build --release
		cbindgen --config cbindgen.toml --crate bufro --output include/bufro.h

install:
		cp target/release/libbufro.so /usr/lib/libbufro.so
		cp include/* /usr/include
