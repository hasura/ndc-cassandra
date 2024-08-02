package com.hasura;

import com.sun.net.httpserver.*;
import java.io.*;
import java.net.*;
    
class App implements HttpHandler {
    public static void main (String[] args) throws IOException {
	HttpServer server = HttpServer.create(new InetSocketAddress(8000), 0);
	server.createContext("/applications/myapp", new App());
	server.setExecutor(null); // creates a default executor
	server.start();
    }

    public void handle(HttpExchange t) throws IOException {
	InputStream is = t.getRequestBody();
	String response = "This is the response";
	t.sendResponseHeaders(200, response.length());
	OutputStream os = t.getResponseBody();
	os.write(response.getBytes());
	os.close();
    }
}
