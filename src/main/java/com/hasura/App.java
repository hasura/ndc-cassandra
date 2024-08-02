package com.hasura;

import java.io.*;
import javax.xml.ws.*;
import javax.xml.ws.http.*;
import javax.xml.transform.*;
import javax.xml.transform.stream.*;

@WebServiceProvider
@ServiceMode(value = Service.Mode.PAYLOAD)
public class App implements Provider<Source> {
    @Override
    public Source invoke(Source request) {
	return new StreamSource(new StringReader("<p>Hello There!</p>"));
    }
    public static void main (String[] args) throws InterruptedException {
	String address = "http://127.0.0.1:8000/";
	Endpoint.create(HTTPBinding.HTTP_BINDING, new App()).publish(address);
	System.out.println("Service running at " + address);
	System.out.println("Type [CTRL]+[C] to quit!");
	Thread.sleep(Long.MAX_VALUE);
    }
}
