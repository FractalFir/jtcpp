// Enables reference counting. This can be used alongside GC quickly to clean up short-lived objects
#define ARC_OBJS
// Enables Bohem GC. This can be disabled if ARC_OBJS is enabled and no cyclical referees are possible.
#define GC_OBJS