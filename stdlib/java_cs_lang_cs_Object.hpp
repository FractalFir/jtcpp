#pragma once
#ifndef java_cs_lang_cs_Object_H
#define java_cs_lang_cs_Object_H 
#include "runtime.hpp"
#include <memory>
struct java_cs_lang_cs_Object: public std::enable_shared_from_this<java_cs_lang_cs_Object>{
      
};
template <typename T> class RuntimeArray : java_cs_lang_cs_Object{
      T* data;
      size_t length;
public:
      RuntimeArray(size_t length);
      T* Get(size_t index);
      void Set(size_t index);
};
#endif
