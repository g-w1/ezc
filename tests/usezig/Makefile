build:
	cargo run -q -- main.ez -nolink -g
	zig build -Drelease-fast
	ld out.o zig-cache/lib/*.a -o out
