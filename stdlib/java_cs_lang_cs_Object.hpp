#pragma once
#include "config.hpp"
#include <bit>
#include <memory>
#include <cmath>  
#include <assert.h>
#include "gc_cpp.h"
#include <cstring>
#ifdef ARC_OBJS
#include <memory>
template<typename T> using ManagedPointer = std::shared_ptr<T>;
template<typename T> inline ManagedPointer<T> managed_from_raw(T* ptr){return std::shared_ptr<T>(ptr);}
#define new_managed(TYPE,ARGS) std::make_shared<TYPE>(ARGS)
#define managed_from_this(TYPE) (std::static_pointer_cast<TYPE>(this->shared_from_this()))
#else
template<typename T> using ManagedPointer = T*;
template<typename T> inline ManagedPointer<T> managed_from_raw(T* ptr){return ptr;}
#define managed_from_this(TYPE) this
#define new_managed(TYPE,ARGS) new TYPE(ARGS)
#endif
namespace java{namespace lang{class Object;};};
class java::lang::Object : public gc
#ifdef ARC_OBJS
,public std::enable_shared_from_this<java::lang::Object>
#endif
{
public:
      static void _init___V(ManagedPointer<java::lang::Object> obj);
};
template <typename T> class RuntimeArray : public java::lang::Object{
      T* data;
      int length;
public:
      RuntimeArray(int length){
            this->data = new T[length];
            this->length = length;
      }
      T Get(int index){
            return this->data[index];
      }
      void Set(int index,T value){
            this->data[index] = value;
      }
      T* GetPtr(int index){
            return &(this->data[index]);
      }
      int GetLength(){
            return this->length;
      }
};
template <> class RuntimeArray<bool>{
      bool* data;
      int length;
public:
      RuntimeArray(int length){
            this->data = new bool[length];
            this->length = length;
      }
      bool Get(int index){
            return this->data[index];
      }
      void Set(int index,bool value){
            this->data[index] = value;
      }
      bool* GetPtr(int index){
            return &(this->data[index]);
      }
      int GetLength(){
            return this->length;
      }
      void Set(int index, uint8_t value){
            this->data[index] = (bool)value;
      }
      void Set(int index, uint16_t value){
            this->data[index] = (bool)value;
      }
      void Set(int index, int value){
            this->data[index] = (bool)value;
      }
};