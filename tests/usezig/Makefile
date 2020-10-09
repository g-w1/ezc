build:
	cargo run -- main.ez -nolink
	zig build -Drelease-fast
	ld out.o zig-cache/lib/*.a -o out
