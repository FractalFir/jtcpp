#include "java_cs_io_cs_PrintStream.hpp"
#include <codecvt>
#include <locale>
#include <string>
java::io::PrintStream::PrintStream(std::unique_ptr<OutuptStreamWrapper> out_stream){
    this->out_stream = std::move(out_stream);
}
void java::io::PrintStream::println_java_cs_lang_cs_String__V(ManagedPointer<java::lang::String> string){
    this->print_java_cs_lang_cs_String__V(string);
    ManagedPointer<RuntimeArray<int8_t>> buffer = managed_from_raw(new RuntimeArray<int8_t>(2));
    buffer->Set(0,'\n');
    buffer->Set(1,'\r');
    this->write__arr_B_V(buffer);
    this->flush__V();
}
void java::io::PrintStream::print_java_cs_lang_cs_String__V(ManagedPointer<java::lang::String> string){
    std::string converted = std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>{}.to_bytes(string->GetBuffer());
    ManagedPointer<RuntimeArray<int8_t>> buffer = managed_from_raw(new RuntimeArray<int8_t>(converted.length()));
    memcpy(buffer->GetPtr(0),converted.data(),buffer->GetLength());
    this->write__arr_B_V(buffer);
}
#include <cstdio>
void java::io::PrintStream::print_F_V(float value){
    uint8_t string[64];
    ManagedPointer<RuntimeArray<int8_t>> buffer = managed_from_raw(new RuntimeArray<int8_t>(64));
    int length = sprintf((char*)&buffer,"%f",value);
    this->write__arr_B_V(buffer);
}
void java::io::PrintStream::print_I_V(int value){
    // Max,  2 147 483 648, log10(2 147 483 648) = 9.3, so 10 digits, 1 sign,+ null so 12 in total
    ManagedPointer<RuntimeArray<int8_t>> buffer = managed_from_raw(new RuntimeArray<int8_t>(12));
    int length = sprintf((char*)&buffer,"%i",value);
    this->write__arr_B_V(buffer);
}
void java::io::PrintStream::println_I_V(int value){
    // Max,  2 147 483 648, log10(2 147 483 648) = 9.3, so 10 digits, 1 sign ,+ \n\r + null, 14 in total
    ManagedPointer<RuntimeArray<int8_t>> buffer = managed_from_raw(new RuntimeArray<int8_t>(14));
    int length = sprintf((char*)&buffer,"%i\n\r",value);
    this->write__arr_B_V(buffer);
    this->flush__V();
}
void java::io::PrintStream::print_C_V(char16_t value){
    char16_t data[] = {value,0};
    std::string converted = std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>{}.to_bytes(data); 
    ManagedPointer<RuntimeArray<int8_t>> buffer = managed_from_raw(new RuntimeArray<int8_t>(converted.length()));
    memcpy(buffer->GetPtr(0),converted.data(),buffer->GetLength());
    this->write__arr_B_V(buffer);
}
void java::io::PrintStream::print_Z_V(bool value){
    if(value){
        this->out_stream->write((int8_t*)"true\n\r",6);
    }
    else{
        this->out_stream->write((int8_t*)"false\n\r",7);
    }
}
void java::io::PrintStream::println__V(){
    this->out_stream->write((int8_t*)"\n\r",2);
    this->flush__V();
}


