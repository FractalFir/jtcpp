#pragma once
#include "java_cs_io_cs_OutputStream.hpp"
namespace java{namespace io{class FilterOutputStream;};};
class java::io::FilterOutputStream:public java::io::OutputStream{
    protected:
        FilterOutputStream();
    public:
        FilterOutputStream(std::unique_ptr<OutuptStreamWrapper> out_stream);
};
