# What is `jtcpp`
`jtcpp` is a versatile, highly experimental JVM bytecode to C++ converter. It generates C++ code which an exact, JVM op into C++ statement, translation of the input bytecode. This approach has some limitations, which may reduce performance of generated C++ code. This makes general comparisons between speed original Java code and translated C++ very hard, because it varies so much(From 3x speedup to 50x slowdown).
# Compatibility.
Building `jtcpp` is supported only on Linux. `make`,`cmake`,`git` and either `g++` or `clang` is required.
Translated `C++` code should work with almost any compiler. It was tested with both `g++` and `clang`, and minimal supported C++ version is C++ 11. 
# Highly versatile GC
`jtcpp` supports 4 different GC modes: No GC, Bohem GC, Reference counting, Mixed-mode GC(experimental, combines reference counting and Bohem GC)
# How to use `jtcpp`
1. Download `jtcpp` and build it.
2. Pick a target directory.
3. Run this command for each file you want to be translated(besides `.class` files, `jtcpp` also supports translating whole `.jar` files in one go)
`jtcpp MY_TARGET_DIR -s JAVA_FILE_1.jar -s JAVA_FILE_2.class`
NOTE:All translated dependencies should have the same target directory
4. Go to your target directory
    b) If you so desire, change `config.hpp` to configure some more advanced features *currently only the way GC works*.
5. run `make -j` and wait as translated `C++` is being built
6. Go to `build` directory within your target directory, `translated.out` is the result of building translated C++ code.
7. On default, `jtcpp` uses Bohem GC. So, `libgc.so` and `libgccpp.so` need to be shipped alongside `translated.out`.
# Java Standard Library
`jtcpp` ships with a minimal, bare-bones implementation of java standard library. The shipped version of the standard library is meant only for testing, and contains only support for classes such as `String`, `Object`, `System` and `PrintStream`, required for outputting to console. Those classes contain only implementations of strictly necessary methods.
# Java features
`jtcpp` supports object creation, 1D arrays, inheritance, static and virtual methods. Support for generics is partial and they may not always work.
`jtcpp` does not support multi dimensional arrays, interfaces, switch statements, exception handling.
# JVM bytcode Ops 
`jtcpp` currently supports almost all JVM opcodes, besides: `tableswitch`, `lookupswitch` `dup2_x2`, `multanewarray`, `invokedynamic`.

