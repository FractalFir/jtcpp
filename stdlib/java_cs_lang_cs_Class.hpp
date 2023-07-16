#pragma once
#include "java_cs_lang_cs_Object.hpp"
namespace java{namespace lang{class Class;};};
class java::lang::Class: public java::lang::Object{
    public:
        Class(char16_t* name);
};