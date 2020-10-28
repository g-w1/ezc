build:
	cargo run -q -- lib.ez -lib
	cargo run -q -- main.ez -nolink
	ld main.ez.o lib.ez.o ../../lib/zig-cache/lib/libstd.a -o out
