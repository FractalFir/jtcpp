#pragma once
#include "java_cs_io_cs_FilterOutputStream.hpp"
#include "java_cs_lang_cs_String.hpp"
namespace java{namespace io{class PrintStream;};};
class java::io::PrintStream:public java::io::FilterOutputStream{
    public:
        PrintStream(OutuptStreamWrapper* out_stream);
        virtual void println_java_cs_lang_cs_String__V(java::lang::String* string);
        virtual void print_java_cs_lang_cs_String__V(java::lang::String* string);
        virtual void print_F_V(float out_float);
};
