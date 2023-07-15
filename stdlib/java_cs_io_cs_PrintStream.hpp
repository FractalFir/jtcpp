#pragma once
#include "java_cs_io_cs_FilterOutputStream.hpp"
#include "java_cs_lang_cs_String.hpp"
namespace java{namespace io{class PrintStream;};};
class java::io::PrintStream:public java::io::FilterOutputStream{
    public:
        PrintStream(OutuptStreamWrapper* out_stream);

        virtual void print_java_cs_lang_cs_String__V(ManagedPointer<java::lang::String> string);
        virtual void print_I_V(int value);
        virtual void print_F_V(float value);
        virtual void print_C_V(char16_t value);
        virtual void print_Z_V(bool value);

        virtual void println_java_cs_lang_cs_String__V(ManagedPointer<java::lang::String> string);
        virtual void println_I_V(int value);
        virtual void println__V();

};
