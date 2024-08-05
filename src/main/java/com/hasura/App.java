package com.hasura;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.util.concurrent.Executors;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpServer;

class App {
    public static void main (String[] args) throws IOException {
	var response = "This is the response".getBytes();
	var objectMapper = new ObjectMapper();
	HttpServer server = HttpServer.create(new InetSocketAddress(8000), 0);
	server
	    .setExecutor(Executors.newVirtualThreadPerTaskExecutor());
	server
	    .createContext("/", exchange -> {
		    exchange.sendResponseHeaders(200, response.length);
		    try (var os = exchange.getResponseBody()) {
			os.write(response);}});
	server
	    .createContext("/healthz",
			   exchange -> {
			       exchange
				   .sendResponseHeaders(204, 0);});
	server
	    .createContext("/metrics",
			   exchange -> {
			       exchange
				   .getResponseHeaders()
				   .add("content-type", "text/plain");
			       exchange
				   .sendResponseHeaders(200, 0);
			       exchange
				   .getResponseBody()
				   .write("""
					  # HELP active_requests number of active requests
					  # TYPE active_requests gauge
					  active_requests 1
					  # HELP total_requests number of total requests
					  # TYPE total_requests counter
					  total_requests 48
					  """
					  .getBytes());});
	server
	    .createContext("/capabilities",
			   exchange -> {
			       new Capabilities(
				   new QueryCapabilities(
				       new LeafCapability (), 
				       null,
				       null, 
				       null), 
				   new MutationCapabilities(
				       null, 
				       null), 
				   null);
			   });
	server.start();}}

