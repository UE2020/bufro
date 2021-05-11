CC=gcc
CFLAGS=-g -O2 -lbufro -lX11 -lGL -lGLU -lglfw -lm
TARGET=quickstart

$(TARGET): main.c
	$(CC) main.c $(CFLAGS) -o $(TARGET)

clean:
	$(RM) $(TARGET)
