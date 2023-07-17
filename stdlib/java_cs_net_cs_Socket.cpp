#include "java_cs_net_cs_Socket.hpp"
#include <unistd.h>
java::net::Socket::Socket(int connection_fd,ManagedPointer<java::net::ServerSocket> parrent_socket){
    this->connection_fd = connection_fd;
    this->parrent_socket = parrent_socket;
}
java::net::Socket::~Socket(){
    this->close__V();
}
void java::net::Socket::close__V(){
    close(this->connection_fd);
}
ManagedPointer<java::io::OutputStream> java::net::Socket::getOutputStream__java_cs_io_cs_OutputStream_(){
    return managed_from_raw(new java::io::OutputStream(new FdWriter(this->connection_fd)));
}