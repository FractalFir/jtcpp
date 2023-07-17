#pragma once
#include "java_cs_io_cs_OutputStream.hpp"
#include "java_cs_lang_cs_Object.hpp"
namespace java{namespace net{class Socket;};};
namespace java{namespace net{class ServerSocket;};};
class java::net::Socket: public java::lang::Object{
    int connection_fd;
    ManagedPointer<java::net::ServerSocket> parrent_socket;
    public:
    virtual ~Socket();
    Socket(int connection_fd,ManagedPointer<java::net::ServerSocket> parrent_socket);
    virtual ManagedPointer<java::io::OutputStream> getOutputStream__java_cs_io_cs_OutputStream_();
    virtual void close__V();
};