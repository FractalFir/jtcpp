#pragma once
#include <memory>
#include <cmath>  
#include <assert.h>
//TEMPORARY!!
struct gc{};
namespace java{namespace lang{class Object;};};
class java::lang::Object : gc{
public:
      static void _init___V(java::lang::Object* obj);
};
template <typename T> class RuntimeArray : java::lang::Object{
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
