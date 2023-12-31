#include "java_cs_lang_cs_Object.hpp"

#define CONCAT_(x,y) x##y
#define CONCAT(x,y) CONCAT_(x,y)
#define CREATE_LAMBDA_CLASS(FUNCTION_NAME,RETURN_TYPE,FUNCTION_ARGS) \
struct LAMBDA_CLASS_##FUNCTION_NAME : java::lang::Object{\
    virtual RETURN_TYPE FUNCTION_NAME FUNCTION_ARGS = 0; \
    virtual ~LAMBDA_CLASS_##FUNCTION_NAME() = 0;\
}; 
/*
#define CREATE_LAMBDA_CLASS_IMPL(FUNCTION_NAME,FUNCTION_IMPL,RETURN_TYPE,FUNCTION_ARGS,LAMBDA_CAPTURE_TYPES,LAMBDA_CAPTURE) \
struct CONCAT(LAMBDA_CLASS##_##FUNCTION_NAME##_IMPL_,__LINE__) : LAMBDA_CLASS_##FUNCTION_NAME{ \
    CONCAT(LAMBDA_CLASS##_##FUNCTION_NAME##_IMPL_,__LINE__)(LAMBDA_CAPTURE_TYPES);\
    virtual RETURN_TYPE FUNCTION_NAME(FUNCTION_ARGS) = FUNCTION_IMPL;\
    virtual ~CONCAT(LAMBDA_CLASS##_##FUNCTION_NAME##_IMPL_,__LINE__)() = default;\
}; \ 
ManagedPointer<LAMBDA_CLASS_##FUNCTION_NAME> lambda##__LINE__ = managed_from_raw(new CONCAT(LAMBDA_CLASS##_##FUNCTION_NAME##_IMPL_,__LINE__)(LAMBDA_CAPTURE)); \
lambda##__LINE__
*/


