CC = cc
CFLAGS = -I../../include -shared -fPIC
LDFLAGS = -L../../zig-out/lib -Wl,-rpath,../../zig-out/lib -lnyx -lraylib

SRC = main.c
OUT = libnyxraylib.dylib

$(OUT): $(SRC)
	$(CC) $(CFLAGS) $(SRC) -o $(OUT) $(LDFLAGS)

clean:
	rm -f $(OUT)
