ffi:
		cargo build --release
		cbindgen --config cbindgen.toml --crate bufro --output bufro.h
		cd c_examples && g++ main.cpp -g -lGL -lGLU -lX11 -lXrandr -lglfw -L ../target/release -lbufro
install:
		mv target/release/libbufro.so /usr/lib
