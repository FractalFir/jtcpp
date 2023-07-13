#include "java_cs_io_cs_PrintStream.hpp"
java::io::PrintStream::PrintStream(OutuptStreamWrapper* out_stream){
    this->out_stream = out_stream;
}