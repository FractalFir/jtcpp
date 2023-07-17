#pragma once
#include "java_cs_lang_cs_Object.hpp"
#include <stdio.h>
#define BUFFER_CAP 1024
class OutuptStreamWrapper{
protected:
    size_t buff_offset;
    int8_t buffer[BUFFER_CAP];  
public:
    virtual void close() = 0;
    virtual void flush() = 0;
    virtual void write(int8_t* buffer,size_t byte_count);
    virtual ~OutuptStreamWrapper() = default;
};
class StdOut: public OutuptStreamWrapper{
public:
    virtual void close();
    virtual void flush();
    virtual ~StdOut() = default;
};
//Writes to a file descriptor.
class FdWriter: public OutuptStreamWrapper{
    FILE* fd;
    virtual void close();
    virtual void flush();
public:
    virtual ~FdWriter() = default;
    FdWriter(int fd);
};
namespace java{namespace io{class OutputStream;};};
class java::io::OutputStream:public java::lang::Object{
protected:
    std::unique_ptr<OutuptStreamWrapper> out_stream;
    OutputStream();
public:
    OutputStream(OutuptStreamWrapper* out_stream);
    virtual void close__V();
    virtual void flush__V();
    virtual void write__arr_B_V(ManagedPointer<RuntimeArray<int8_t>> arr);
    virtual void write__arr_BII_V(ManagedPointer<RuntimeArray<int8_t>> arr,int off, int len);
    virtual void write_I_V(int byte);
};
class StreamChain: public OutuptStreamWrapper{
    ManagedPointer<java::io::OutputStream> stream;
    virtual void close();
    virtual void flush();
public:
    StreamChain(ManagedPointer<java::io::OutputStream> stream);
    virtual ~StreamChain() = default;
};
