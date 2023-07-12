#pragma once
#include "java_cs_lang_cs_Object.hpp"
#include <ostream>
class java_cs_io_cs_OutputStream:java_cs_lang_cs_Object{
    protected:
        std::ostream out_stream;
    public:
        java_cs_io_cs_OutputStream(std::ostream out_stream);
        virtual void close();
        virtual void flush();
        virtual void write(RuntimeArray<uint8_t>* arr);
        virtual void write(RuntimeArray<uint8_t>* arr,int off, int len);
        virtual void write(int byte);
};
