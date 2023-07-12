#include "java_cs_lang_cs_Object.hpp"
#ifndef java_cs_lang_cs_String_H
#define java_cs_lang_cs_String_H 
class java_cs_lang_cs_String : java_cs_lang_cs_Object{
    char16_t* buffer;
    size_t length;
    public:
    java_cs_lang_cs_String(const char16_t* buffer,size_t length);
    java_cs_lang_cs_String(const char16_t* null_terminated_buffer);
};
#endif

