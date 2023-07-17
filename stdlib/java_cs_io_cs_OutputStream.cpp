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
void OutuptStreamWrapper::write(int8_t* buffer, size_t length){
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
void java::io::OutputStream::flush__V(){
    this->out_stream->flush();
}
void java::io::OutputStream::close__V(){
    this->out_stream->close();
}
void java::io::OutputStream::write__arr_BII_V(ManagedPointer<RuntimeArray<int8_t>> arr,int off, int len){
    assert(off + len < arr->GetLength());
    int8_t *buffer = arr->GetPtr(off);
    this->out_stream->write(buffer,len);
}
void java::io::OutputStream::write__arr_B_V(ManagedPointer<RuntimeArray<int8_t>> arr){
    this->write__arr_BII_V(arr,0,arr->GetLength());
}
void java::io::OutputStream::write_I_V(int byte_int){
    int8_t byte = (int8_t)byte_int;
    this->out_stream->write(&byte,1);
}
java::io::OutputStream::OutputStream(){}
java::io::OutputStream::OutputStream(OutuptStreamWrapper* out_stream){
    this->out_stream = std::unique_ptr<OutuptStreamWrapper>(out_stream);
}
FdWriter::FdWriter(int fd){
    this->fd = (FILE*)(size_t)fd;
}
void FdWriter::close(){}
void FdWriter::flush(){
    fwrite(this->buffer,sizeof (unsigned char), this->buff_offset, this->fd);
    this->buff_offset = 0;
}