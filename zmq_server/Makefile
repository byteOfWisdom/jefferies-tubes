CFLAGS=-std=c++23 -O2 -Wall -Wextra -Wpedantic
ZMQLIB=$(shell pkg-config --cflags cppzmq)
ZMQLINK=$(shell pkg-config --libs cppzmq)

all:
	$(CXX) $(CFLAGS) $(ZMQLIB) -o server  main.cpp

clean:
	rm server
