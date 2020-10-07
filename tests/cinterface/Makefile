build:
	cargo run -q -- lib.ez -c
	gcc main.c out.o -o out
	gcc ref.c -o ref
clean:
	rm *out*
	rm *.o
