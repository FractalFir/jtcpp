#pragma once
#include "java_cs_lang_cs_Object.hpp"
namespace java{namespace lang{class String;};};
class java::lang::String : java::lang::Object{
    char16_t* buffer;
    size_t length;
    public:
    char16_t* GetBuffer();
    size_t GetBufferLength();
    String(const char16_t* buffer,size_t length);
    String(const char16_t* null_terminated_buffer);
};

