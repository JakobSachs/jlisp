cc=gcc
CFLAGS=$(shell pkg-config --cflags libedit)
LDFLAGS=$(shell pkg-config --libs libedit)

jlisp: jlisp.c
	$(cc) $(CFLAGS) -g -fno-omit-frame-pointer -std=c17 -Wall -o jlisp jlisp.c $(LDFLAGS) -Lmpc/build -lmpc -Wl,-rpath,mpc/build

.PHONY: clean
clean:
	rm jlisp
