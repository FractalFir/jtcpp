#pragma once
#ifndef RUNTIME_GUARD
#define RUNTIME_GUARD 
#include <stddef.h>
#include <math.h>
typedef struct ClassData{} ClassData;
void* get_virtual();
void* runtime_alloc_new(size_t size);
#endif
