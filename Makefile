cc=gcc
CFLAGS=$(shell pkg-config --cflags libedit)
LDFLAGS=$(shell pkg-config --libs libedit)

jlisp: jlisp.c
	$(cc) $(CFLAGS) -std=c17 -Wall -o jlisp jlisp.c $(LDFLAGS) -Lmpc/build -lmpc -Wl,-rpath,mpc/build
