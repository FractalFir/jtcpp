#pragma once
#include "java_cs_io_cs_FilterOutputStream.hpp"
#include "java_cs_lang_cs_String.hpp"
class java_cs_io_cs_PrintStream:public java_cs_io_cs_FilterOutputStream{
    public:
        java_cs_io_cs_PrintStream(OutuptStreamWrapper* out_stream);
        virtual void println_ne__ab_java_cs_lang_cs_String_as_ae_V(java_cs_lang_cs_String* string);
        virtual void print_ne__ab_java_cs_lang_cs_String_as_ae_V(java_cs_lang_cs_String* string);
        virtual void print_ne__ab_Fae_V(float out_float);
};
