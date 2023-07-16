#include "java_cs_io_cs_OutputStream.hpp"
#define min(a, b) (((a) < (b)) ? (a) : (b))
#include <stdio.h>
#include <cstring>
#include <assert.h>
void StdOut::close(){}
void StdOut::flush(){
    fwrite(this->buffer,sizeof (unsigned char), this->buff_offset, stdout);
    this->buff_offset = 0;
}
void OutuptStreamWrapper::write(uint8_t* buffer, size_t length){
    if (this->buff_offset + length >= BUFFER_CAP) this->flush();
    if (length < BUFFER_CAP){
        memcpy(this->buffer + this->buff_offset,buffer,length);
        this->buff_offset += length;
        assert(this->buff_offset < BUFFER_CAP);
    }
    else{
        
        while(length > 0){
            size_t curr_length = min(length,BUFFER_CAP);
            this->write(buffer,curr_length);
            buffer += curr_length;
            length -= curr_length;
        }
    }
}
void java::io::OutputStream::flush_ne__ab__as_ae_V(){
    this->out_stream->flush();
}
void java::io::OutputStream::close_ne__ab__as_ae_V(){
    this->out_stream->close();
}
void java::io::OutputStream::write(RuntimeArray<uint8_t>* arr,int off, int len){
    assert(off + len < arr->GetLength());
    uint8_t *buffer = arr->GetPtr(off);
    this->out_stream->write(buffer,len);
}
void java::io::OutputStream::write(RuntimeArray<uint8_t>* arr){
    this->write(arr,0,arr->GetLength());
}
void java::io::OutputStream::write(int byte_int){
    uint8_t byte = (uint8_t)byte_int;
    this->out_stream->write(&byte,1);
}
java::io::OutputStream::OutputStream(){}