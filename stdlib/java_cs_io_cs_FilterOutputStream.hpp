#pragma once
#include "java_cs_io_cs_OutputStream.hpp"
class java_cs_io_cs_FilterOutputStream:java_cs_io_cs_OutputStream{
    public:
        java_cs_io_cs_FilterOutputStream(std::ostream out_stream);
};
