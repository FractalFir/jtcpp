#pragma once
#include "java_cs_lang_cs_Object.hpp"
#define BUFFER_CAP 1024
class OutuptStreamWrapper{
protected:
    size_t buff_offset;
    uint8_t buffer[BUFFER_CAP];  
public:
    virtual void close() = 0;
    virtual void flush() = 0;
    virtual void write(uint8_t* buffer,size_t byte_count);
};
class StdOut: public OutuptStreamWrapper{
    virtual void close();
    virtual void flush();
};
namespace java{namespace io{class OutputStream;};};
class java::io::OutputStream:public java::lang::Object{
    protected:
        OutuptStreamWrapper* out_stream;
        OutputStream();
    public:
        OutputStream(OutuptStreamWrapper* out_stream);
        virtual void close_ne__ab__as_ae_V();
        virtual void flush_ne__ab__as_ae_V();
        virtual void write(RuntimeArray<uint8_t>* arr);
        virtual void write(RuntimeArray<uint8_t>* arr,int off, int len);
        virtual void write(int byte);
};
