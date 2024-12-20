CVLIBS		:= ""
CURLPPLIBS	:= ""
build: Main.o
	g++ -o bin/main obj/Main.o -Wall\
		-Llib/x86_64-linux-gnu -lcurlpp -Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -Wl,-z,relro -Wl,-z,now -lcurl
Main.o: src/main.cpp src/main.hpp
	g++ src/main.cpp -o obj/Main.o -Wall -g -c -I/usr/local/include/opencv4\
		-Iinclude -I/usr/include/x86_64-linux-gnu
init:
	mkdir obj bin