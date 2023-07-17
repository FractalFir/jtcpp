import java.net.Socket;
import java.net.ServerSocket;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
class HTTPServer{
  //private static final int port = 1234;
  public static void main(String[] args) throws java.io.IOException{
    int port = 1234;
    ServerSocket server = new ServerSocket(port);
    while(true){
      System.out.println("Waiting for a HTTP client request");
      Socket socket = server.accept();
      System.out.println("Got a request!");
      OutputStream stream = socket.getOutputStream();
      String message = "HTTP/1.1 200 OK\nContent-Type: text/html\n\n<html><head><title>This server was in Java, is now in C++!</title></head><body><h1> Java To C++</h1><br>This is a java HTML server, converted into C++, and compiled to native instructions.</body></html>";
      byte bytes[] = message.getBytes(StandardCharsets.UTF_8);
      System.out.println("Sending a lovely message to client!");
      stream.write(bytes);
      stream.close();
      socket.close();
    }
  }
}
