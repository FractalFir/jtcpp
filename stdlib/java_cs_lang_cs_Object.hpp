#pragma once
#ifndef java_cs_lang_cs_Object_H
#define java_cs_lang_cs_Object_H 
#include "runtime.hpp"
#include <memory>
struct java_cs_lang_cs_Object: public std::enable_shared_from_this<java_cs_lang_cs_Object>{
      virtual void _init__ne__ab_ae_V();
};
template <typename T> class RuntimeArray : java_cs_lang_cs_Object{
      T* data;
      int length;
public:
      RuntimeArray(int length);
      T* Get(int index);
      void Set(int index,T data);
};
#endif
