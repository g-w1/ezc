build:
	cargo run -q -- lib.ez -c
	gcc main.c out.o -O3 -o out
clean:
	rm *out*
	rm *.o
debug:
	cargo run -q -- lib.ez -c -g
	gcc -g main.c out.o -o out
