#pragma once
#include "java_cs_lang_cs_Object.hpp"
#include "java_cs_net_cs_Socket.hpp"
namespace java{namespace net{class ServerSocket;};};
class java::net::ServerSocket: public java::lang::Object{
    int socket_fd = 0;
    public:
        static void _init__I_V(ManagedPointer<ServerSocket> socket,int socket_port);
        virtual ManagedPointer<java::net::Socket> accept__java_cs_net_cs_Socket_(); 
        virtual void close__V();
};
