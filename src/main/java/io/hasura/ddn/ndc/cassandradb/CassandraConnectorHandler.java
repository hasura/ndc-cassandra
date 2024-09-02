package io.hasura.ddn.ndc.cassandradb;

import io.hasura.ddn.ndc.*;
import io.hasura.ddn.ndc.Models.Capabilities;
import io.hasura.ddn.ndc.Models.ExplainResponse;
import io.hasura.ddn.ndc.Models.MutationRequest;
import io.hasura.ddn.ndc.Models.MutationResponse;
import io.hasura.ddn.ndc.Models.QueryRequest;
import io.hasura.ddn.ndc.Models.QueryResponse;
import io.hasura.ddn.ndc.Models.SchemaResponse;

public class CassandraConnectorHandler implements ConnectorHandler {

    @Override
    public Capabilities getCapabilities() {
	// TODO Auto-generated method stub
	throw new UnsupportedOperationException("Unimplemented method 'getCapabilities'");
    }

    @Override
    public SchemaResponse getSchema() {
	// TODO Auto-generated method stub
	throw new UnsupportedOperationException("Unimplemented method 'getSchema'");
    }

    @Override
    public MutationResponse mutation(MutationRequest request) {
	// TODO Auto-generated method stub
	throw new UnsupportedOperationException("Unimplemented method 'mutation'");
    }

    @Override
    public ExplainResponse mutationExplain(MutationRequest request) {
	// TODO Auto-generated method stub
	throw new UnsupportedOperationException("Unimplemented method 'mutationExplain'");
    }

    @Override
    public QueryResponse query(QueryRequest arg0) {
	// TODO Auto-generated method stub
	throw new UnsupportedOperationException("Unimplemented method 'query'");
    }

    @Override
    public ExplainResponse queryExplain(QueryRequest arg0) {
	// TODO Auto-generated method stub
	throw new UnsupportedOperationException("Unimplemented method 'queryExplain'");
    }}
