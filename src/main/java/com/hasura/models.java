package com.hasura;

import java.util.List;
import java.util.Map;
import jakarta.annotation.Nullable;

record Foo(String bar) {}

record CapabilitiesResponse(
			    String version,
			    Capabilities capabilities) {}

record Capabilities(
		    QueryCapabilities query,
		    MutationCapabilities mutation,
		    @Nullable RelationshipCapabilities relationships) {}

record QueryCapabilities(
			 @Nullable LeafCapability aggregates,
			 @Nullable LeafCapability variables,
			 @Nullable LeafCapability explain,
			 @Nullable NestedFieldCapabilities nested_fields) {}

record LeafCapability(
) {}

record NestedFieldCapabilities(
			       @Nullable LeafCapability filter_by,
			       @Nullable LeafCapability order_by,
			       @Nullable LeafCapability aggregates) {}

record MutationCapabilities(
			    @Nullable LeafCapability transactional,
			    @Nullable LeafCapability explain) {}

record RelationshipCapabilities(
				@Nullable LeafCapability relation_comparisons,
				@Nullable LeafCapability order_by_aggregate) {}

record SchemaResponse(
		      Map<String, Object> scalar_types,
		      Map<String, Object> object_types,
		      List<CollectionInfo> collections,
		      List<FunctionInfo> functions,
		      List<ProcedureInfo> procedures) {}

record ScalarType(
		  @Nullable TypeRepresentation representation,
		  Map<String, Object> aggregate_functions,
		  Map<String, Object> comparison_operators) {}

record TypeRepresentation(
) {}

record AggregateFunctionDefinition(
				   Type result_type) {}

enum Type {
    NAMED,
    NULLABLE,
    ARRAY,
    PREDICATE
}

// record Type(
// ) {}

record ComparisonOperatorDefinition(
) {}

record ObjectType(
		  @Nullable String description,
		  Map<String, Object> fields) {}

record ObjectField(
		   @Nullable String description,
		   Type type,
		   Map<String, Object> arguments) {}

record ArgumentInfo(
		    @Nullable String description,
		    Type type) {}

record CollectionInfo(
		      String name,
		      @Nullable String description,
		      Map<String, Object> arguments,
		      String type,
		      Map<String, Object> uniqueness_constraints,
		      Map<String, Object> foreign_keys) {}

record UniquenessConstraint(
			    List<String> unique_columns) {}

record ForeignKeyConstraint(
			    Map<String, Object> column_mapping,
			    String foreign_collection) {}

record FunctionInfo(
		    String name,
		    @Nullable String description,
		    Map<String, Object> arguments,
		    Type result_type) {}

record ProcedureInfo(
		     String name,
		     @Nullable String description,
		     Map<String, Object> arguments,
		     Type result_type) {}

record QueryRequest(
		    String collection,
		    Query query,
		    Map<String, Object> arguments,
		    Map<String, Object> collection_relationships,
		    @Nullable List<Map<String, Object>> variables) {}

record Query(
	     @Nullable Map<String, Aggregate> aggregates,
	     @Nullable Map<String, Field> fields,
	     @Nullable int limit,
	     @Nullable int offset,
	     @Nullable OrderBy order_by,
	     @Nullable Expression predicate) {}

record Aggregate(
) {}

record Field(
) {}

record NestedField(
) {}

record Argument(
) {}

record RelationshipArgument(
) {}

record OrderBy(
	       List<OrderByElement> elements) {}

record OrderByElement(
		      OrderDirection order_direction,
		      OrderByTarget target) {}

record OrderDirection(
) {}

record OrderByTarget(
) {}

record PathElement(
		   String relationship,
		   Map<String, Object> arguments,
		   @Nullable Expression predicate) {}

record Expression(
) {}

record ComparisonTarget(
) {}

record UnaryComparisonOperator(
) {}

record ComparisonValue(
) {}

record ExistsInCollection(
) {}

record Relationship(
		    Map<String, Object> column_mapping,
		    RelationshipType relationship_type,
		    String target_collection,
		    Map<String, Object> arguments) {}

record RelationshipType(
) {}

record QueryResponse(
) {}

record RowSet(
	      @Nullable Map<String, Object> aggregates,
	      @Nullable List<Map<String, RowFieldValue>> rows) {}

record RowFieldValue(
) {}

record MutationRequest(
		       List<MutationOperation> operations,
		       Map<String, Object> collection_relationships) {}

record MutationOperation(
) {}

record MutationResponse(
			List<MutationOperationResults> operation_results) {}

record MutationOperationResults(
) {}

record ExplainResponse(
		       Map<String, Object> details) {}

record ErrorResponse(
		     String message,
		     Map<String, Object> details) {}

record ValidateResponse(
			SchemaResponse schema,
			CapabilitiesResponse capabilities,
			String resolved_configuration) {}
