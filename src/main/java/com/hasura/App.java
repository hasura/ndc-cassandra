package com.hasura;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.util.concurrent.Executors;

import com.sun.net.httpserver.HttpServer;
    
class App {
    public static void main (String[] args) throws IOException {
	var response = "This is the response".getBytes();
	HttpServer server = HttpServer.create(new InetSocketAddress(8000), 0);
	server.setExecutor(Executors.newVirtualThreadPerTaskExecutor());
	server.createContext("/", exchange -> {
		exchange.sendResponseHeaders(200, response.length);
		try (var os = exchange.getResponseBody()) {
		    os.write(response);}});
	server.start();}}

