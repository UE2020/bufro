CC=g++
CFLAGS=-g -O2 -lbufro -lX11 -lGL -lGLU -lglfw
TARGET=quickstart

$(TARGET): main.cpp
	$(CC) main.cpp $(CFLAGS) -o $(TARGET)

clean:
	$(RM) $(TARGET)
