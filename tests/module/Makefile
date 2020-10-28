build:
	cargo run -- lib.ez -lib
	cargo run -- main.ez -nolink
	ld main.ez.o lib.ez.o ../../lib/zig-cache/lib/libstd.a -o out
