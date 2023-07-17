#include "java_cs_lang_cs_System.hpp"
#include <iostream>
ManagedPointer<java::io::PrintStream> java::lang::System::out = managed_from_raw(new java::io::PrintStream(std::unique_ptr<StdOut>(new StdOut())));