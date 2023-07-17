#pragma once
#include "java_cs_lang_cs_Object.hpp"
#include "java_cs_nio_cs_charset_cs_Charset.hpp"
namespace java{namespace nio{namespace charset{class StandardCharsets;};};};
class java::nio::charset::StandardCharsets: public java::lang::Object{
    public:
    static ManagedPointer<java::nio::charset::Charset> UTF_8;
};