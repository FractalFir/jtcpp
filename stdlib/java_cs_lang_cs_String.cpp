#include "java_cs_lang_cs_String.hpp"
#include <cstring>
#include <codecvt>
java::lang::String::String(const char16_t* buffer,size_t length){
    bool addNull = false;
    if (buffer[length - 1] != 0){
        length += 1;
        addNull = true;
    }
    this->data = std::u16string();
    this->data.resize(length);
    for(int i = 0; i < (int)length; i++){
        this->data[i] = buffer[i];
    }
    if (addNull)this->data[length - 1] = 0;
}
const char16_t* java::lang::String::GetBuffer(){return this->data.data();}
java::lang::String::String(const char16_t* null_terminated_buffer){
    unsigned int length = 0;
    const char16_t* curr = null_terminated_buffer;
    while(*curr){
        curr += 1;
        length += 1;
    }
    this->data = std::u16string();
    this->data.resize(length);
    for(int i = 0; i < (int)length; i++){
        this->data[i] = null_terminated_buffer[i];
    }
}
#include <stdexcept>
java::lang::String::String(std::u16string data){this->data = data;}
ManagedPointer<java::lang::String> java::lang::String::from_cstring(char* cstring){
    size_t length = 0;
    char* curr = cstring;
    bool can_lazy_encode = true;
    while(curr){
        length++;
        //highest bit set, Not ASCII
        if(*curr & 0x80){
            can_lazy_encode = false;
        }
        curr+=1;
    }
    if(can_lazy_encode){
        curr = cstring;
        size_t curr_index = 0;
        std::unique_ptr<char16_t[]> buff = std::unique_ptr<char16_t[]>(new char16_t[length + 1]);
        while(curr){
            buff[curr_index*2] = *curr;
            buff[curr_index*2 + 1] = 0;
            curr_index+=1;
        }
        ManagedPointer<java::lang::String> res = managed_from_raw(new java::lang::String(buff.get()));
        return res;
    }
    //std::codecvt_utf16<char> convert;
    //std::mbstate_t mystate = std::mbstate_t();
    throw new std::runtime_error("Can't yet translate UTF-8 strings, such as CLI args, to UTF-16. Converting UTF-8 to UTF16 in C++ is, sadly, very confusing. :(");
    //return managed_from_raw(new String(u16.data()));
}
