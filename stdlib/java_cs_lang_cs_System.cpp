#include "java_cs_lang_cs_System.hpp"
#include <iostream>
java::io::PrintStream* java::lang::System::out = new java::io::PrintStream(new StdOut());