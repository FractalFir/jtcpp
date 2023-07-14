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