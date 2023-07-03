#pragma once
#ifndef java_cs_lang_cs_Object_H
#define java_cs_lang_cs_Object_H 
#define CURR_VMETHOD_COUNT 0
#include "runtime.h"
//Fielddef BEGINS
#define java_cs_lang_cs_Object_FIELDS \
ClassData* class; \
void** vtable;  
//Fielddef ENDS
 typedef struct java_cs_lang_cs_Object{
      java_cs_lang_cs_Object_FIELDS 
} java_cs_lang_cs_Object;
#endif
#define java_cs_lang_cs_Object_VMETHOD_LAST CURR_VMETHOD_COUNT + 1
