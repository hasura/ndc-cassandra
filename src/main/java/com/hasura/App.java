package com.hasura;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.util.concurrent.Executors;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpServer;
import com.hasura.models.*;

class App {
    public static void main (String[] args) throws IOException {
	var objectMapper = new ObjectMapper();
	HttpServer server = HttpServer.create(new InetSocketAddress(8000), 0);
	server
	    .setExecutor(Executors.newVirtualThreadPerTaskExecutor());
	server
	    .createContext("/healthz",
			   exchange -> {
			       exchange
				   .sendResponseHeaders(204, -1);});
	server
	    .createContext("/metrics",
			   exchange -> {
			       var response = """
				   # HELP active_requests number of active requests
				   # TYPE active_requests gauge
				   active_requests 1
				   # HELP total_requests number of total requests
				   # TYPE total_requests counter
				   total_requests 48
				   """
				   .getBytes();
			       exchange
				   .getResponseHeaders()
				   .add("content-type", "text/plain");
			       exchange
				   .sendResponseHeaders(200, response.length);
			       exchange
				   .getResponseBody()
				   .write(response);});
	server
	    .createContext("/capabilities",
			   exchange -> {
			       var response = objectMapper.writeValueAsBytes(
				   new Capabilities(
				       new QueryCapabilities(
					   new LeafCapability (), 
					   null,
					   null, 
					   null), 
				       new MutationCapabilities(
					   null, 
					   null), 
				       null));
			       exchange
				   .getResponseHeaders()
				   .add("content-type", "application/json");
			       exchange
				   .sendResponseHeaders(200, response.length);
			       exchange
				   .getResponseBody()
				   .write(response);});
	server.start();}}

