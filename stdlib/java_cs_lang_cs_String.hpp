#pragma once
#include "java_cs_lang_cs_Object.hpp"
#include <string>
namespace java{namespace lang{class String;};};
class java::lang::String : public java::lang::Object{
    std::u16string data;
    public:
    const char16_t* GetBuffer();
    size_t GetBufferLength();
    String(const char16_t* buffer,size_t length);
    String(const char16_t* null_terminated_buffer);
    String(std::u16string data);
    static ManagedPointer<String> from_cstring(char* cstring);
};

