build:
	cargo run -q -- lib.ez -lib
	gcc main.c lib.ez.o -O3 -o out
clean:
	rm *out*
	rm *.o
