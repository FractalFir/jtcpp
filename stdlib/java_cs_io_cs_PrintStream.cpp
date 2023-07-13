#include "java_cs_io_cs_PrintStream.hpp"
#include <codecvt>
#include <locale>
//#include <string>
java::io::PrintStream::PrintStream(OutuptStreamWrapper* out_stream){
    this->out_stream = out_stream;
}
void java::io::PrintStream::println_java_cs_lang_cs_String__V(java::lang::String* string){
    this->print_java_cs_lang_cs_String__V(string);
    this->out_stream->write((uint8_t*)"\n\r",2);
    this->out_stream->flush();
}
void java::io::PrintStream::print_java_cs_lang_cs_String__V(java::lang::String* string){
    std::string converted = std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>{}.to_bytes(string->GetBuffer()); 
    this->out_stream->write((uint8_t*) converted.c_str(),converted.length());
}
#include <cstdio>
void java::io::PrintStream:: print_F_V(float out_float){
    uint8_t string[64];
    int length = sprintf((char*)&string,"%f",out_float);
    this->out_stream->write(string,length);
}
