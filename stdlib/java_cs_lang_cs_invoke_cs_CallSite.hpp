#pragma once
#include "java_cs_lang_cs_Object.hpp"
#include "java_cs_lang_cs_invoke_MethodHandle.hpp"
namespace java{namespace lang{namespace invoke{class CallSite;};};};
class java::lang::invoke::CallSite:java::lang::Object{
    MethodHandle* handle;
public:
    virtual MethodHandle* getTarget() = 0;
    virtual void setTarget(MethodHandle*) = 0;
};