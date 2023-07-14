#pragma once
#include "java_cs_lang_cs_Object.hpp"
#include "java_cs_io_cs_PrintStream.hpp"
namespace java{namespace lang{class System;};};
class java::lang::System:public java::lang::Object{
    public:
        static java::io::PrintStream* out;// = new java_cs_io_cs_PrintStream(get_std_out())
};