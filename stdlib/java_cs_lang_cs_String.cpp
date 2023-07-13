#include "java_cs_lang_cs_String.hpp"
#include <cstring>
java::lang::String::String(const char16_t* buffer,size_t length){
    char16_t* new_buffer = new char16_t[length];
    std::memcpy(new_buffer,buffer,length*sizeof(char16_t));
    this->buffer = new_buffer;
    this->length = length;
}
java::lang::String::String(const char16_t* null_terminated_buffer){
    unsigned int length = 0;
    const char16_t* curr = null_terminated_buffer;
    while(*curr){
        curr += 1;
        length += 1;
    }
    char16_t* new_buffer = new char16_t[length];
    std::memcpy(new_buffer,buffer,length*sizeof(char16_t));
    this->buffer = new_buffer;
    this->length = length;
}