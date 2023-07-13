#include "java_cs_lang_cs_Object.hpp"
#ifndef java_cs_lang_cs_String_H
#define java_cs_lang_cs_String_H 
namespace java{namespace lang{class String;};};
class java::lang::String : java::lang::Object{
    char16_t* buffer;
    size_t length;
    public:
    String(const char16_t* buffer,size_t length);
    String(const char16_t* null_terminated_buffer);
};
#endif

