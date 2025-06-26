//! GraphQL Federation tests for Knowledge Graph Service

use knowledge_graph::api::graphql::create_test_schema;

#[cfg(test)]
mod federation_schema_tests {
    use super::*;

    #[tokio::test]
    async fn test_federation_schema_extension() {
        // Test 6: Federation Schema Extension Test
        // Arrange
        let schema = create_test_schema();
        
        // Act
        let sdl = schema.sdl();
        
        // Assert - verify federation support is present
        assert!(
            sdl.contains("_service: _Service!"),
            "Schema should contain _service query for federation"
        );
        assert!(
            sdl.contains("entities(representations: [JSON!]!): [JSON]!") ||
            sdl.contains("entities(representations: [JSON!]!): [JSON]") ||
            sdl.contains("entities("),
            "Schema should contain entities query for federation"
        );
        assert!(
            sdl.contains("type _Service"),
            "Schema should contain _Service type"
        );
        assert!(
            sdl.contains("scalar _Any"),
            "Schema should contain _Any scalar for federation"
        );
    }

    #[tokio::test]
    async fn test_service_sdl_query() {
        // Test 9: _service Query Test
        use async_graphql::{Request, Variables};
        
        // Arrange
        let schema = create_test_schema();
        let query = r#"
            query {
                _service {
                    sdl
                }
            }
        "#;
        
        // Act
        let request = Request::new(query).variables(Variables::default());
        let response = schema.execute(request).await;
        
        // Assert
        assert!(response.is_ok(), "Service SDL query should succeed");
        
        let data = response.data.into_json().unwrap();
        let sdl = data
            .get("_service")
            .and_then(|s| s.get("sdl"))
            .and_then(|s| s.as_str())
            .expect("SDL should be present in response");
        
        // Verify the SDL contains expected federation directives
        assert!(sdl.contains("extend schema"), "SDL should contain schema extension");
        assert!(sdl.contains("@link"), "SDL should contain @link directive");
        assert!(
            sdl.contains("federation/v2") || sdl.contains("federation/v2.0") || sdl.contains("federation/v2.5"), 
            "SDL should reference federation v2.x"
        );
        assert!(sdl.contains("@key"), "SDL should import @key directive");
        
        // Verify the SDL contains expected Knowledge Graph types
        assert!(sdl.contains("type Concept"), "SDL should contain Concept type");
        assert!(sdl.contains("type LearningResource"), "SDL should contain LearningResource type");
        assert!(sdl.contains("type User"), "SDL should contain User type");
    }
}

#[cfg(test)]
mod federation_entity_tests {
    use super::*;
    use async_graphql::{Request, Variables, Value, Name};
    use serde_json::json;

    #[tokio::test]
    async fn test_entity_resolution_concept() {
        // Test 7: Concept Entity Federation Test
        // Arrange
        let schema = create_test_schema();
        let representations = json!([
            {
                "__typename": "Concept",
                "id": "concept-123"
            }
        ]);
        
        let query = r#"
            query ResolveEntities($representations: [JSON!]!) {
                entities(representations: $representations) {
                    ... on Concept {
                        __typename
                        id
                        name
                        description
                    }
                    ... on LearningResource {
                        __typename
                        id
                        title
                    }
                    ... on User {
                        __typename
                        id
                    }
                    ... on UserProgress {
                        __typename
                        userId
                    }
                    ... on Workflow {
                        __typename
                        id
                    }
                }
            }
        "#;
        
        // Act
        let mut variables = Variables::default();
        variables.insert(Name::new("representations"), Value::from_json(representations).unwrap());
        let request = Request::new(query).variables(variables);
        let response = schema.execute(request).await;
        
        // Assert
        if !response.is_ok() {
            println!("Response errors: {:?}", response.errors);
        }
        assert!(response.is_ok(), "Entity resolution query should succeed");
        
        let data = response.data.into_json().unwrap();
        let entities = data
            .get("entities")
            .and_then(|e| e.as_array())
            .expect("entities should return an array");
        
        assert_eq!(entities.len(), 1, "Should resolve one entity");
        
        let entity = entities[0].as_object().expect("Entity should be an object");
        assert_eq!(
            entity.get("__typename").and_then(|t| t.as_str()),
            Some("Concept"),
            "Entity should have correct __typename"
        );
        assert_eq!(
            entity.get("id").and_then(|i| i.as_str()),
            Some("concept-123"),
            "Entity should have correct id"
        );
        assert!(entity.contains_key("name"), "Entity should have name field");
        assert!(entity.contains_key("description"), "Entity should have description field");
    }

    #[tokio::test]
    async fn test_entity_resolution_learning_resource() {
        // Test 8: LearningResource Entity Federation Test
        // Arrange
        let schema = create_test_schema();
        let representations = json!([
            {
                "__typename": "LearningResource",
                "id": "resource-456"
            }
        ]);
        
        let query = r#"
            query ResolveEntities($representations: [JSON!]!) {
                entities(representations: $representations) {
                    ... on Concept {
                        __typename
                        id
                        name
                        description
                    }
                    ... on LearningResource {
                        __typename
                        id
                        title
                    }
                    ... on User {
                        __typename
                        id
                    }
                    ... on UserProgress {
                        __typename
                        userId
                    }
                    ... on Workflow {
                        __typename
                        id
                    }
                }
            }
        "#;
        
        // Act
        let mut variables = Variables::default();
        variables.insert(Name::new("representations"), Value::from_json(representations).unwrap());
        let request = Request::new(query).variables(variables);
        let response = schema.execute(request).await;
        
        // Assert
        assert!(response.is_ok(), "Entity resolution query should succeed");
        
        let data = response.data.into_json().unwrap();
        let entities = data
            .get("entities")
            .and_then(|e| e.as_array())
            .expect("entities should return an array");
        
        assert_eq!(entities.len(), 1, "Should resolve one entity");
        
        let entity = entities[0].as_object().expect("Entity should be an object");
        assert_eq!(
            entity.get("__typename").and_then(|t| t.as_str()),
            Some("LearningResource"),
            "Entity should have correct __typename"
        );
        assert_eq!(
            entity.get("id").and_then(|i| i.as_str()),
            Some("resource-456"),
            "Entity should have correct id"
        );
        assert!(entity.contains_key("title"), "Entity should have title field");
    }

    #[tokio::test]
    async fn test_entity_resolution_multiple_entities() {
        // Test 10: _entities Query Test (Multiple Entities)
        // Arrange
        let schema = create_test_schema();
        let representations = json!([
            {
                "__typename": "Concept",
                "id": "concept-1"
            },
            {
                "__typename": "LearningResource",
                "id": "resource-1"
            },
            {
                "__typename": "Concept",
                "id": "concept-2"
            }
        ]);
        
        let query = r#"
            query ResolveEntities($representations: [JSON!]!) {
                entities(representations: $representations) {
                    ... on Concept {
                        __typename
                        id
                        name
                        description
                    }
                    ... on LearningResource {
                        __typename
                        id
                        title
                    }
                    ... on User {
                        __typename
                        id
                    }
                    ... on UserProgress {
                        __typename
                        userId
                    }
                    ... on Workflow {
                        __typename
                        id
                    }
                }
            }
        "#;
        
        // Act
        let mut variables = Variables::default();
        variables.insert(Name::new("representations"), Value::from_json(representations).unwrap());
        let request = Request::new(query).variables(variables);
        let response = schema.execute(request).await;
        
        // Assert
        assert!(response.is_ok(), "Entity resolution query should succeed");
        
        let data = response.data.into_json().unwrap();
        let entities = data
            .get("entities")
            .and_then(|e| e.as_array())
            .expect("entities should return an array");
        
        assert_eq!(entities.len(), 3, "Should resolve three entities");
        
        // Check first entity
        let entity0 = entities[0].as_object().expect("Entity 0 should be an object");
        assert_eq!(
            entity0.get("__typename").and_then(|t| t.as_str()),
            Some("Concept"),
            "First entity should be Concept"
        );
        
        // Check second entity
        let entity1 = entities[1].as_object().expect("Entity 1 should be an object");
        assert_eq!(
            entity1.get("__typename").and_then(|t| t.as_str()),
            Some("LearningResource"),
            "Second entity should be LearningResource"
        );
        
        // Check third entity
        let entity2 = entities[2].as_object().expect("Entity 2 should be an object");
        assert_eq!(
            entity2.get("__typename").and_then(|t| t.as_str()),
            Some("Concept"),
            "Third entity should be Concept"
        );
    }

    #[tokio::test]
    async fn test_entity_resolution_external_entities() {
        // Test external entities (should be returned as-is)
        // Arrange
        let schema = create_test_schema();
        let representations = json!([
            {
                "__typename": "User",
                "id": "user-789"
            },
            {
                "__typename": "Workflow",
                "id": "workflow-101"
            }
        ]);
        
        let query = r#"
            query ResolveEntities($representations: [JSON!]!) {
                entities(representations: $representations) {
                    ... on Concept {
                        __typename
                        id
                        name
                        description
                    }
                    ... on LearningResource {
                        __typename
                        id
                        title
                    }
                    ... on User {
                        __typename
                        id
                    }
                    ... on UserProgress {
                        __typename
                        userId
                    }
                    ... on Workflow {
                        __typename
                        id
                    }
                }
            }
        "#;
        
        // Act
        let mut variables = Variables::default();
        variables.insert(Name::new("representations"), Value::from_json(representations).unwrap());
        let request = Request::new(query).variables(variables);
        let response = schema.execute(request).await;
        
        // Assert
        assert!(response.is_ok(), "Entity resolution query should succeed");
        
        let data = response.data.into_json().unwrap();
        let entities = data
            .get("entities")
            .and_then(|e| e.as_array())
            .expect("entities should return an array");
        
        assert_eq!(entities.len(), 2, "Should resolve two entities");
        
        // External entities should be returned as-is (pass-through)
        let entity0 = entities[0].as_object().expect("Entity 0 should be an object");
        assert_eq!(
            entity0.get("__typename").and_then(|t| t.as_str()),
            Some("User"),
            "First entity should be User"
        );
        assert_eq!(
            entity0.get("id").and_then(|i| i.as_str()),
            Some("user-789"),
            "User should have correct id"
        );
        
        let entity1 = entities[1].as_object().expect("Entity 1 should be an object");
        assert_eq!(
            entity1.get("__typename").and_then(|t| t.as_str()),
            Some("Workflow"),
            "Second entity should be Workflow"
        );
        assert_eq!(
            entity1.get("id").and_then(|i| i.as_str()),
            Some("workflow-101"),
            "Workflow should have correct id"
        );
    }

    #[tokio::test]
    async fn test_entity_resolution_invalid_entity() {
        // Test invalid entity handling
        // Arrange
        let schema = create_test_schema();
        let representations = json!([
            {
                "__typename": "InvalidType",
                "id": "invalid-1"
            },
            {
                "__typename": "Concept",
                "id": "concept-1"
            }
        ]);
        
        let query = r#"
            query ResolveEntities($representations: [JSON!]!) {
                entities(representations: $representations) {
                    ... on Concept {
                        __typename
                        id
                        name
                        description
                    }
                    ... on LearningResource {
                        __typename
                        id
                        title
                    }
                    ... on User {
                        __typename
                        id
                    }
                    ... on UserProgress {
                        __typename
                        userId
                    }
                    ... on Workflow {
                        __typename
                        id
                    }
                }
            }
        "#;
        
        // Act
        let mut variables = Variables::default();
        variables.insert(Name::new("representations"), Value::from_json(representations).unwrap());
        let request = Request::new(query).variables(variables);
        let response = schema.execute(request).await;
        
        // Assert
        assert!(response.is_ok(), "Entity resolution query should succeed even with invalid types");
        
        let data = response.data.into_json().unwrap();
        let entities = data
            .get("entities")
            .and_then(|e| e.as_array())
            .expect("entities should return an array");
        
        assert_eq!(entities.len(), 2, "Should return two results");
        
        // First entity should be null (invalid type)
        assert!(entities[0].is_null(), "Invalid entity should return null");
        
        // Second entity should be resolved
        let entity1 = entities[1].as_object().expect("Entity 1 should be an object");
        assert_eq!(
            entity1.get("__typename").and_then(|t| t.as_str()),
            Some("Concept"),
            "Second entity should be Concept"
        );
    }
}