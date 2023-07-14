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
void java::io::PrintStream::print_F_V(float value){
    uint8_t string[64];
    int length = sprintf((char*)&string,"%f",value);
    this->out_stream->write(string,length);
}
void java::io::PrintStream::print_I_V(int value){
    // Max, 2 147 483 648, so 10 digits,+ null, 11 in total
    uint8_t string[11];
    int length = sprintf((char*)&string,"%i",value);
    this->out_stream->write(string,length);
}
void java::io::PrintStream::println_I_V(int value){
    // Max,  2 147 483 648, so 10 digits,+ \n\r + null, 13 in total
    uint8_t string[13];
    int length = sprintf((char*)&string,"%i\n\r",value);
    this->out_stream->write(string,length);
    this->out_stream->flush();
}
void java::io::PrintStream::print_C_V(char16_t value){
    char16_t data[] = {value,0};
    std::string converted = std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>{}.to_bytes(data); 
    this->out_stream->write((uint8_t*) converted.c_str(),converted.length());
}
void java::io::PrintStream::print_Z_V(bool value){
    if(value){
        this->out_stream->write((uint8_t*)"true\n\r",6);
    }
    else{
        this->out_stream->write((uint8_t*)"false\n\r",7);
    }
}
void java::io::PrintStream::println__V(){
    this->out_stream->write((uint8_t*)"\n\r",2);
    this->out_stream->flush();
}


