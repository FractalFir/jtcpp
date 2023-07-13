#pragma once
#include "java_cs_io_cs_OutputStream.hpp"
class java_cs_io_cs_FilterOutputStream:public java_cs_io_cs_OutputStream{
    protected:
        java_cs_io_cs_FilterOutputStream();
    public:
        java_cs_io_cs_FilterOutputStream(std::ofstream out_stream);
};
