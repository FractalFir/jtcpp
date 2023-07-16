MYDIR = .
CC = g++
CFLAGS = -Ofast -flto 
LINKER = g++
all: prepare_build compile_sources link
prepare_build:
	mkdir -p build
compile_sources: $(MYDIR)/*.cpp
	$(foreach file, $(wildcard $(MYDIR)/*.cpp), $(CC) $(CFLAGS) -c ${file} -o "build/${file}.o";) 
link:
	cd build ;\
	$(LINKER) -flto *.o -L. -lgc -lgccpp -o translated.out
