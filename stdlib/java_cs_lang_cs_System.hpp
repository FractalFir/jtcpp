#include "java_cs_lang_cs_Object.hpp"
#include "java_cs_io_cs_PrintStream.hpp"
class java_cs_lang_cs_System:java_cs_lang_cs_Object{
    public:
        static java_cs_io_cs_PrintStream* out;// = new java_cs_io_cs_PrintStream(get_std_out())
};