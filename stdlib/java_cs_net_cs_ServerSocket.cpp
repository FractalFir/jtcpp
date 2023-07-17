#include "java_cs_net_cs_ServerSocket.hpp"
#include "java_cs_net_cs_Socket.hpp"
#include <sys/socket.h>
#include <netinet/in.h>
#include <stdio.h>
#include <unistd.h>
void java::net::ServerSocket::_init__I_V(ManagedPointer<ServerSocket> server_socket,int socket_port){
    int socket_fd = socket(AF_INET,SOCK_STREAM | SOCK_CLOEXEC,0); 
    struct sockaddr_in serv_addr;
    //Clear adress structure
    memset(&serv_addr,sizeof(serv_addr),0);
    serv_addr.sin_family = AF_INET;  
    serv_addr.sin_addr.s_addr = INADDR_ANY; 
    serv_addr.sin_port = htons(socket_port);
    if(bind(socket_fd,(const struct sockaddr*)&serv_addr,sizeof(serv_addr))){
        fprintf(stderr,"Could not open a new socket. TODO: Throw an exception instead of aborting!");
        abort();
    }
    listen(socket_fd,32);
    server_socket->socket_fd = socket_fd;
}
ManagedPointer<java::net::Socket> java::net::ServerSocket::accept__java_cs_net_cs_Socket_(){
    struct sockaddr_in connection_addr;
    uint connection_adress_len = sizeof(connection_addr);
    int connection_fd = accept(this->socket_fd,(struct sockaddr*)&connection_addr,&connection_adress_len);
    return managed_from_raw(new Socket(connection_fd,managed_from_this(ServerSocket)));
}
void java::net::ServerSocket::close__V(){
    close(this->socket_fd);
}