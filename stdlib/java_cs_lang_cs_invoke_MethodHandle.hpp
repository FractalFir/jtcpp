#pragma once
#include "java_cs_lang_cs_Object.hpp"
namespace java{namespace lang{namespace invoke{class MethodHandle;};};};
class java::lang::invoke::MethodHandle:java::lang::Object{
    public:
    virtual java::lang::Object* invokeExact(RuntimeArray<java::lang::Object*>* args) = 0;
};