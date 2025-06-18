//! DGraph GraphQL response parsing
//! 
//! Provides utilities for parsing GraphQL responses from DGraph
//! into domain objects with support for nested structures,
//! aliases, and fragments.

use crate::error::{KnowledgeGraphError, Result, ErrorContext, ResultExt, OperationFailure, partial_result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::time::Instant;

use crate::graph::{Concept, LearningResource, LearningPath, UserProgress};

/// GraphQL response parser for DGraph
pub struct DgraphResponseParser;

impl DgraphResponseParser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Self
    }

    /// Parse a query result into a typed response
    pub fn parse_query_result<T>(&self, response: Value) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Parsing GraphQL response: {:?}", response);

        // Check for GraphQL errors
        if let Some(errors) = response.get("errors") {
            if let Some(errors_array) = errors.as_array() {
                if !errors_array.is_empty() {
                    let graphql_errors = self.parse_graphql_errors(errors_array);
                    let error_messages: Vec<String> = graphql_errors
                        .iter()
                        .map(|e| e.message.clone())
                        .collect();
                    
                    return Err(KnowledgeGraphError::GraphQLError {
                        message: error_messages.join(", "),
                        errors: graphql_errors,
                        query: None,
                    });
                }
            }
        }

        // Extract data field
        let data = response
            .get("data")
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing 'data' field in GraphQL response".to_string(),
                field: Some("data".to_string()),
                raw_data: Some(response.to_string()),
                source_error: None,
            })?;

        // Deserialize the data
        serde_json::from_value(data.clone())
            .map_err(|e| KnowledgeGraphError::ParseError {
                message: format!("Failed to deserialize GraphQL response: {}", e),
                field: None,
                raw_data: Some(data.to_string()),
                source_error: Some(e.to_string()),
            })
    }

    /// Parse a single concept from a GetConcept query result
    pub fn parse_concept_from_result(&self, response: Value) -> Result<Concept> {
        debug!("Parsing concept from result");

        let data = self.extract_data(&response)?;
        
        // Handle both direct concept response and nested concept field
        let concept_data = if let Some(concept) = data.get("concept") {
            concept
        } else if let Some(get_concept) = data.get("getConcept") {
            get_concept
        } else {
            &data
        };

        self.parse_concept(concept_data)
    }

    /// Parse concepts from a search result with graceful degradation
    pub fn parse_concepts_from_search_result(&self, response: Value, limit: Option<usize>) -> Result<Vec<Concept>> {
        debug!("Parsing concepts from search result");

        let data = self.extract_data(&response)?;
        
        // Handle different possible field names
        let concepts_data = data.get("concepts")
            .or_else(|| data.get("queryConcept"))
            .or_else(|| data.get("searchConcepts"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "No concepts field found in search result".to_string(),
                field: Some("concepts".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;

        let concepts_array = concepts_data
            .as_array()
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Concepts field is not an array".to_string(),
                field: Some("concepts".to_string()),
                raw_data: Some(concepts_data.to_string()),
                source_error: None,
            })?;

        let mut concepts = Vec::new();
        let mut failures = Vec::new();
        let max_items = limit.unwrap_or(concepts_array.len());

        for (i, concept_data) in concepts_array.iter().enumerate() {
            if i >= max_items {
                break;
            }
            
            match self.parse_concept(concept_data) {
                Ok(concept) => concepts.push(concept),
                Err(e) => {
                    warn!("Failed to parse concept at index {}: {}", i, e);
                    failures.push(OperationFailure {
                        operation_id: format!("parse_concept_{}", i),
                        error_message: e.to_string(),
                        error_type: "ParseError".to_string(),
                        timestamp: Instant::now(),
                    });
                }
            }
        }

        // If we have some results but also failures, return partial results
        if !concepts.is_empty() && !failures.is_empty() {
            error!("Partial parsing failure: {} concepts parsed, {} failed", concepts.len(), failures.len());
            // For now, we'll just log this but still return the successful results
            // In a more strict mode, we could return a PartialResultError
        }

        // If all parsing failed, return an error
        if concepts.is_empty() && !failures.is_empty() {
            return Err(partial_result(
                "Failed to parse any concepts from search result",
                0,
                failures.len(),
                None::<Vec<Concept>>,
                failures,
            ));
        }

        Ok(concepts)
    }

    /// Parse concepts from a graph traversal result
    pub fn parse_concepts_from_graph_result(&self, response: Value) -> Result<Vec<Concept>> {
        debug!("Parsing concepts from graph result");

        let data = self.extract_data(&response)?;
        let mut all_concepts = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // Parse the main concept
        if let Some(concept_data) = data.get("concept") {
            if let Ok(concept) = self.parse_concept(concept_data) {
                if seen_ids.insert(concept.id) {
                    all_concepts.push(concept.clone());
                }

                // Parse related concepts from various relationship fields
                let relationship_fields = ["prerequisites", "enabledBy", "relatedTo", "subtopics"];
                
                for field in &relationship_fields {
                    if let Some(related_data) = concept_data.get(field) {
                        if let Some(related_array) = related_data.as_array() {
                            for related_concept_data in related_array {
                                if let Ok(related_concept) = self.parse_concept(related_concept_data) {
                                    if seen_ids.insert(related_concept.id) {
                                        all_concepts.push(related_concept);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(all_concepts)
    }

    /// Parse a mutation result
    pub fn parse_mutation_result(&self, response: Value) -> Result<MutationResult> {
        debug!("Parsing mutation result: {:?}", response);

        let data = self.extract_data(&response)?;
        
        // Handle different mutation response formats
        // Single operations
        if let Some(add_concept) = data.get("addConcept") {
            self.parse_add_mutation_result(add_concept)
        } else if let Some(update_concept) = data.get("updateConcept") {
            self.parse_update_mutation_result(update_concept)
        } else if let Some(delete_concept) = data.get("deleteConcept") {
            self.parse_delete_mutation_result(delete_concept)
        }
        // Bulk operations
        else if let Some(add_concepts) = data.get("addConcepts") {
            self.parse_bulk_add_mutation_result(add_concepts)
        } else if let Some(update_concepts) = data.get("updateConcepts") {
            self.parse_bulk_update_mutation_result(update_concepts)
        } else if let Some(delete_concepts) = data.get("deleteConcepts") {
            self.parse_bulk_delete_mutation_result(delete_concepts)
        }
        // Generic mutation response (e.g., from raw mutations)
        else if let Some(mutation_response) = data.get("mutate") {
            self.parse_raw_mutation_result(mutation_response)
        }
        // Check for other resource types
        else if let Some(add_resource) = data.get("addLearningResource") {
            self.parse_add_resource_mutation_result(add_resource)
        } else if let Some(add_path) = data.get("addLearningPath") {
            self.parse_add_path_mutation_result(add_path)
        } else if let Some(add_progress) = data.get("addUserProgress") {
            self.parse_add_progress_mutation_result(add_progress)
        } else {
            // Try to extract any mutation-like response
            self.parse_generic_mutation_result(&data)
        }
    }

    /// Parse learning resources from a response
    pub fn parse_learning_resources(&self, response: Value) -> Result<Vec<LearningResource>> {
        debug!("Parsing learning resources");

        let data = self.extract_data(&response)?;
        let resources_data = data.get("resources")
            .or_else(|| data.get("queryResource"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "No resources field found".to_string(),
                field: Some("resources".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;

        let resources_array = resources_data
            .as_array()
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Resources field is not an array".to_string(),
                field: Some("resources".to_string()),
                raw_data: Some(resources_data.to_string()),
                source_error: None,
            })?;

        let mut resources = Vec::new();
        for resource_data in resources_array {
            if let Ok(resource) = self.parse_learning_resource(resource_data) {
                resources.push(resource);
            }
        }

        Ok(resources)
    }

    /// Parse a learning path from a response
    pub fn parse_learning_path(&self, response: Value) -> Result<LearningPath> {
        debug!("Parsing learning path");

        let data = self.extract_data(&response)?;
        let path_data = data.get("learningPath")
            .or_else(|| data.get("path"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "No learning path field found".to_string(),
                field: Some("learningPath".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;

        self.parse_path(path_data)
    }

    /// Parse user progress from a response
    pub fn parse_user_progress(&self, response: Value) -> Result<Vec<UserProgress>> {
        debug!("Parsing user progress");

        let data = self.extract_data(&response)?;
        let progress_data = data.get("progress")
            .or_else(|| data.get("userProgress"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "No progress field found".to_string(),
                field: Some("progress".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;

        let progress_array = progress_data
            .as_array()
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Progress field is not an array".to_string(),
                field: Some("progress".to_string()),
                raw_data: Some(progress_data.to_string()),
                source_error: None,
            })?;

        let mut progress_items = Vec::new();
        for item_data in progress_array {
            if let Ok(progress) = self.parse_progress(item_data) {
                progress_items.push(progress);
            }
        }

        Ok(progress_items)
    }

    /// Handle GraphQL aliases in responses
    pub fn resolve_aliases(&self, response: &mut Value) -> Result<()> {
        debug!("Resolving GraphQL aliases");

        if let Some(data) = response.get_mut("data").and_then(|d| d.as_object_mut()) {
            let mut aliased_fields = HashMap::new();

            // Detect aliased fields (fields with colons in GraphQL)
            for (key, value) in data.iter() {
                if key.contains(':') {
                    if let Some(alias) = key.split(':').next() {
                        aliased_fields.insert(alias.to_string(), value.clone());
                    }
                }
            }

            // Merge aliased fields back
            for (alias, value) in aliased_fields {
                data.insert(alias, value);
            }
        }

        Ok(())
    }

    /// Handle GraphQL fragments in responses
    pub fn expand_fragments(&self, response: &mut Value, fragments: &HashMap<String, Value>) -> Result<()> {
        debug!("Expanding GraphQL fragments");

        // Recursively expand fragment spreads
        self.expand_fragments_recursive(response, fragments)
    }

    // Private helper methods

    fn extract_data<'a>(&self, response: &'a Value) -> Result<&'a Value> {
        // Check for errors first
        if let Some(errors) = response.get("errors") {
            if let Some(errors_array) = errors.as_array() {
                if !errors_array.is_empty() {
                    let graphql_errors = self.parse_graphql_errors(errors_array);
                    let error_messages: Vec<String> = graphql_errors
                        .iter()
                        .map(|e| e.message.clone())
                        .collect();
                    
                    return Err(KnowledgeGraphError::GraphQLError {
                        message: error_messages.join(", "),
                        errors: graphql_errors,
                        query: None,
                    });
                }
            }
        }

        response
            .get("data")
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing 'data' field in GraphQL response".to_string(),
                field: Some("data".to_string()),
                raw_data: Some(response.to_string()),
                source_error: None,
            })
    }

    fn parse_graphql_errors(&self, errors: &[Value]) -> Vec<crate::error::GraphQLErrorDetail> {
        errors
            .iter()
            .filter_map(|error| {
                let message = error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                
                let path = error.get("path")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    });
                
                let extensions = error.get("extensions")
                    .and_then(|e| e.as_object())
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    });
                
                Some(crate::error::GraphQLErrorDetail {
                    message,
                    path,
                    extensions,
                })
            })
            .collect()
    }

    fn parse_concept(&self, data: &Value) -> Result<Concept> {
        let context = ErrorContext::new("parse_concept")
            .with_metadata("data_type", "Concept");

        let id = self.parse_uuid(data.get("id"))
            .with_context(|| context.clone().with_metadata("field", "id"))?;
        let name = self.parse_string(data.get("name"), "name")
            .with_context(|| context.clone().with_metadata("field", "name"))?;
        let description = self.parse_optional_string(data.get("description"));
        let difficulty = self.parse_string(data.get("difficulty"), "difficulty")
            .with_context(|| context.clone().with_metadata("field", "difficulty"))?;
        let category = self.parse_string(data.get("category"), "category")
            .with_context(|| context.clone().with_metadata("field", "category"))?;
        let subcategory = self.parse_optional_string(data.get("subcategory"));
        let tags = self.parse_string_array(data.get("tags"));
        let quality_score = self.parse_f32(data.get("qualityScore"), 0.5)
            .with_context(|| context.clone().with_metadata("field", "qualityScore"))?;
        let estimated_time = self.parse_optional_f32(data.get("estimatedTime"));
        let embeddings = self.parse_f32_array(data.get("embeddings"));
        let created_at = self.parse_datetime(data.get("createdAt"))
            .with_context(|| context.clone().with_metadata("field", "createdAt"))?;
        let updated_at = self.parse_datetime(data.get("updatedAt"))
            .with_context(|| context.clone().with_metadata("field", "updatedAt"))?;
        let version = self.parse_i32(data.get("version"), 1)
            .with_context(|| context.clone().with_metadata("field", "version"))?;

        Ok(Concept {
            id,
            name,
            description,
            difficulty,
            category,
            subcategory,
            tags,
            quality_score,
            estimated_time,
            embeddings,
            created_at,
            updated_at,
            version,
        })
    }

    fn parse_learning_resource(&self, data: &Value) -> Result<LearningResource> {
        let id = self.parse_uuid(data.get("id"))?;
        let url = self.parse_string(data.get("url"), "url")?;
        let title = self.parse_string(data.get("title"), "title")?;
        let resource_type = self.parse_string(data.get("resourceType"), "resourceType")?;
        let format = self.parse_optional_string(data.get("format"));
        let source = self.parse_optional_string(data.get("source"));
        let quality = self.parse_optional_f32(data.get("quality"));
        let difficulty = self.parse_optional_string(data.get("difficulty"));
        let duration = self.parse_optional_i32(data.get("duration"));
        let language = self.parse_optional_string(data.get("language"));
        let created_at = self.parse_datetime(data.get("createdAt"))?;
        let updated_at = self.parse_datetime(data.get("updatedAt"))?;

        Ok(LearningResource {
            id,
            url,
            title,
            resource_type,
            format,
            source,
            quality,
            difficulty,
            duration,
            language,
            created_at,
            updated_at,
        })
    }

    fn parse_path(&self, data: &Value) -> Result<LearningPath> {
        let id = self.parse_uuid(data.get("id"))?;
        let name = self.parse_string(data.get("name"), "name")?;
        let description = self.parse_optional_string(data.get("description"));
        let target_audience = self.parse_optional_string(data.get("targetAudience"));
        let estimated_time = self.parse_optional_f32(data.get("estimatedTime"));
        let difficulty_progression = self.parse_optional_string(data.get("difficultyProgression"));
        let learning_outcomes = self.parse_string_array(data.get("learningOutcomes"));
        let creator = self.parse_optional_string(data.get("creator"));
        let is_custom = self.parse_bool(data.get("isCustom"), false)?;
        let created_at = self.parse_datetime(data.get("createdAt"))?;
        let updated_at = self.parse_datetime(data.get("updatedAt"))?;

        Ok(LearningPath {
            id,
            name,
            description,
            target_audience,
            estimated_time,
            difficulty_progression,
            learning_outcomes,
            creator,
            is_custom,
            created_at,
            updated_at,
        })
    }

    fn parse_progress(&self, data: &Value) -> Result<UserProgress> {
        let id = self.parse_uuid(data.get("id"))?;
        let user_id = self.parse_string(data.get("userId"), "userId")?;
        let concept_id = self.parse_uuid(data.get("conceptId"))?;
        let status = self.parse_string(data.get("status"), "status")?;
        let percent_complete = self.parse_optional_f32(data.get("percentComplete"));
        let time_spent = self.parse_optional_i32(data.get("timeSpent"));
        let resources_completed = self.parse_optional_i32(data.get("resourcesCompleted"));
        let difficulty_rating = self.parse_optional_f32(data.get("difficultyRating"));
        let notes = self.parse_optional_string(data.get("notes"));
        let started_at = self.parse_optional_datetime(data.get("startedAt"));
        let completed_at = self.parse_optional_datetime(data.get("completedAt"));
        let last_accessed_at = self.parse_datetime(data.get("lastAccessedAt"))?;

        Ok(UserProgress {
            id,
            user_id,
            concept_id,
            status,
            percent_complete,
            time_spent,
            resources_completed,
            difficulty_rating,
            notes,
            started_at,
            completed_at,
            last_accessed_at,
        })
    }

    fn parse_add_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let concept_data = data.get("concept")
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing concept in add mutation result".to_string(),
                field: Some("concept".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;
        
        let concept = self.parse_concept(concept_data)?;
        
        // Extract additional metadata if available
        let num_uids = self.parse_optional_i32(data.get("numUids")).unwrap_or(1);
        let uid = self.parse_optional_string(data.get("uid"));
        
        Ok(MutationResult {
            success: true,
            message: Some("Concept created successfully".to_string()),
            affected_ids: vec![concept.id],
            affected_count: num_uids as usize,
            operation_type: MutationOperationType::Add,
            data: Some(serde_json::to_value(concept)?),
            uids: uid.map(|u| vec![u]).unwrap_or_default(),
            conflicts: vec![],
        })
    }

    fn parse_update_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let concept_data = data.get("concept")
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing concept in update mutation result".to_string(),
                field: Some("concept".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;
        
        let concept = self.parse_concept(concept_data)?;
        
        // Extract additional metadata if available
        let num_uids = self.parse_optional_i32(data.get("numUids")).unwrap_or(1);
        let conflicts = self.parse_conflict_info(data.get("conflicts"));
        
        Ok(MutationResult {
            success: true,
            message: Some("Concept updated successfully".to_string()),
            affected_ids: vec![concept.id],
            affected_count: num_uids as usize,
            operation_type: MutationOperationType::Update,
            data: Some(serde_json::to_value(concept)?),
            uids: vec![],
            conflicts,
        })
    }

    fn parse_delete_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let msg = self.parse_optional_string(data.get("msg"))
            .unwrap_or_else(|| "Deletion completed".to_string());
        
        let num_uids = self.parse_i32(data.get("numUids"), 0)?;
        
        // Try to extract deleted UIDs if available
        let deleted_uids = self.parse_string_array(data.get("deletedUids"));
        let deleted_ids = self.parse_uuid_array(data.get("deletedIds"));
        
        Ok(MutationResult {
            success: num_uids > 0,
            message: Some(msg),
            affected_ids: deleted_ids,
            affected_count: num_uids as usize,
            operation_type: MutationOperationType::Delete,
            data: None,
            uids: deleted_uids,
            conflicts: vec![],
        })
    }

    fn parse_bulk_add_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let concepts_data = data.get("concept")
            .or_else(|| data.get("concepts"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing concepts in bulk add mutation result".to_string(),
                field: Some("concepts".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;
        
        let concepts_array = concepts_data.as_array()
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Concepts field is not an array".to_string(),
                field: Some("concepts".to_string()),
                raw_data: Some(concepts_data.to_string()),
                source_error: None,
            })?;
        
        let mut affected_ids = Vec::new();
        let mut all_concepts = Vec::new();
        
        for concept_data in concepts_array {
            if let Ok(concept) = self.parse_concept(concept_data) {
                affected_ids.push(concept.id);
                all_concepts.push(concept);
            }
        }
        
        let num_uids = self.parse_optional_i32(data.get("numUids"))
            .unwrap_or(affected_ids.len() as i32);
        
        Ok(MutationResult {
            success: !affected_ids.is_empty(),
            message: Some(format!("Created {} concepts successfully", affected_ids.len())),
            affected_ids,
            affected_count: num_uids as usize,
            operation_type: MutationOperationType::Bulk,
            data: Some(serde_json::to_value(all_concepts)?),
            uids: vec![],
            conflicts: vec![],
        })
    }

    fn parse_bulk_update_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let concepts_data = data.get("concept")
            .or_else(|| data.get("concepts"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing concepts in bulk update mutation result".to_string(),
                field: Some("concepts".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;
        
        let concepts_array = concepts_data.as_array()
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Concepts field is not an array".to_string(),
                field: Some("concepts".to_string()),
                raw_data: Some(concepts_data.to_string()),
                source_error: None,
            })?;
        
        let mut affected_ids = Vec::new();
        let mut all_concepts = Vec::new();
        let mut all_conflicts = Vec::new();
        
        for concept_data in concepts_array {
            if let Ok(concept) = self.parse_concept(concept_data) {
                affected_ids.push(concept.id);
                all_concepts.push(concept);
            }
        }
        
        // Parse any conflict information
        if let Some(conflicts_data) = data.get("conflicts") {
            all_conflicts = self.parse_conflict_info(Some(conflicts_data));
        }
        
        let num_uids = self.parse_optional_i32(data.get("numUids"))
            .unwrap_or(affected_ids.len() as i32);
        
        Ok(MutationResult {
            success: !affected_ids.is_empty(),
            message: Some(format!("Updated {} concepts successfully", affected_ids.len())),
            affected_ids,
            affected_count: num_uids as usize,
            operation_type: MutationOperationType::Bulk,
            data: Some(serde_json::to_value(all_concepts)?),
            uids: vec![],
            conflicts: all_conflicts,
        })
    }

    fn parse_bulk_delete_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let msg = self.parse_optional_string(data.get("msg"))
            .unwrap_or_else(|| "Bulk deletion completed".to_string());
        
        let num_uids = self.parse_i32(data.get("numUids"), 0)?;
        let deleted_uids = self.parse_string_array(data.get("deletedUids"));
        let deleted_ids = self.parse_uuid_array(data.get("deletedIds"));
        
        Ok(MutationResult {
            success: num_uids > 0,
            message: Some(format!("{} (deleted {} items)", msg, num_uids)),
            affected_ids: deleted_ids,
            affected_count: num_uids as usize,
            operation_type: MutationOperationType::Bulk,
            data: None,
            uids: deleted_uids,
            conflicts: vec![],
        })
    }

    fn parse_raw_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        // Handle raw Dgraph mutation response
        let code = self.parse_optional_string(data.get("code"))
            .unwrap_or_else(|| "Success".to_string());
        
        let message = self.parse_optional_string(data.get("message"))
            .unwrap_or_else(|| "Mutation completed".to_string());
        
        let uids = if let Some(uids_obj) = data.get("uids").and_then(|v| v.as_object()) {
            uids_obj.values()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        };
        
        let queries = data.get("queries");
        
        Ok(MutationResult {
            success: code == "Success",
            message: Some(message),
            affected_ids: vec![],
            affected_count: uids.len(),
            operation_type: MutationOperationType::Add,
            data: queries.cloned(),
            uids,
            conflicts: vec![],
        })
    }

    fn parse_add_resource_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let resource_data = data.get("learningResource")
            .or_else(|| data.get("resource"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing resource in add mutation result".to_string(),
                field: Some("resource".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;
        
        let resource = self.parse_learning_resource(resource_data)?;
        
        Ok(MutationResult {
            success: true,
            message: Some("Learning resource created successfully".to_string()),
            affected_ids: vec![resource.id],
            affected_count: 1,
            operation_type: MutationOperationType::Add,
            data: Some(serde_json::to_value(resource)?),
            uids: vec![],
            conflicts: vec![],
        })
    }

    fn parse_add_path_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let path_data = data.get("learningPath")
            .or_else(|| data.get("path"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing path in add mutation result".to_string(),
                field: Some("path".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;
        
        let path = self.parse_path(path_data)?;
        
        Ok(MutationResult {
            success: true,
            message: Some("Learning path created successfully".to_string()),
            affected_ids: vec![path.id],
            affected_count: 1,
            operation_type: MutationOperationType::Add,
            data: Some(serde_json::to_value(path)?),
            uids: vec![],
            conflicts: vec![],
        })
    }

    fn parse_add_progress_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        let progress_data = data.get("userProgress")
            .or_else(|| data.get("progress"))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Missing progress in add mutation result".to_string(),
                field: Some("progress".to_string()),
                raw_data: Some(data.to_string()),
                source_error: None,
            })?;
        
        let progress = self.parse_progress(progress_data)?;
        
        Ok(MutationResult {
            success: true,
            message: Some("User progress created successfully".to_string()),
            affected_ids: vec![progress.id],
            affected_count: 1,
            operation_type: MutationOperationType::Add,
            data: Some(serde_json::to_value(progress)?),
            uids: vec![],
            conflicts: vec![],
        })
    }

    fn parse_generic_mutation_result(&self, data: &Value) -> Result<MutationResult> {
        // For generic mutations, the actual data might be nested under a custom field
        // Try to find the actual mutation data
        let mutation_data = if let Some(obj) = data.as_object() {
            // If there's only one key and it's an object, use that as the mutation data
            if obj.len() == 1 {
                if let Some((_, value)) = obj.iter().next() {
                    if value.is_object() {
                        value
                    } else {
                        data
                    }
                } else {
                    data
                }
            } else {
                data
            }
        } else {
            data
        };
        
        // Try to extract common mutation fields
        let success = self.parse_bool(mutation_data.get("success"), true)?;
        let message = self.parse_optional_string(mutation_data.get("message"))
            .or_else(|| self.parse_optional_string(mutation_data.get("msg")));
        
        let num_uids = self.parse_optional_i32(mutation_data.get("numUids")).unwrap_or(0);
        
        // Try to find any IDs or UIDs
        let mut uids = Vec::new();
        let mut affected_ids = Vec::new();
        
        if let Some(uid) = self.parse_optional_string(mutation_data.get("uid")) {
            uids.push(uid);
        }
        
        if let Some(id_val) = mutation_data.get("id") {
            if let Ok(id) = self.parse_uuid(Some(id_val)) {
                affected_ids.push(id);
            }
        }
        
        // Determine operation type from message or other clues
        let operation_type = if message.as_ref().map_or(false, |m| m.contains("add") || m.contains("create")) {
            MutationOperationType::Add
        } else if message.as_ref().map_or(false, |m| m.contains("update") || m.contains("modify")) {
            MutationOperationType::Update
        } else if message.as_ref().map_or(false, |m| m.contains("delete") || m.contains("remove")) {
            MutationOperationType::Delete
        } else {
            MutationOperationType::Upsert
        };
        
        let affected_count = num_uids.max(affected_ids.len() as i32) as usize;
        
        Ok(MutationResult {
            success,
            message,
            affected_ids,
            affected_count,
            operation_type,
            data: Some(mutation_data.clone()),
            uids,
            conflicts: vec![],
        })
    }

    fn parse_conflict_info(&self, value: Option<&Value>) -> Vec<ConflictInfo> {
        value
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|conflict| {
                        let field = self.parse_optional_string(conflict.get("field"))?;
                        Some(ConflictInfo {
                            field,
                            existing_value: conflict.get("existingValue").cloned(),
                            attempted_value: conflict.get("attemptedValue").cloned(),
                            resolution: self.parse_optional_string(conflict.get("resolution")),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_uuid_array(&self, value: Option<&Value>) -> Vec<Uuid> {
        value
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        v.as_str()
                            .and_then(|s| Uuid::parse_str(s).ok())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn expand_fragments_recursive(&self, value: &mut Value, fragments: &HashMap<String, Value>) -> Result<()> {
        match value {
            Value::Object(map) => {
                let mut expansions = Vec::new();
                
                // Look for fragment spreads
                for (key, _val) in map.iter() {
                    if key.starts_with("...") {
                        if let Some(fragment_name) = key.strip_prefix("...") {
                            if let Some(fragment) = fragments.get(fragment_name) {
                                expansions.push(fragment.clone());
                            }
                        }
                    }
                }
                
                // Apply expansions
                for expansion in expansions {
                    if let Some(expansion_obj) = expansion.as_object() {
                        for (k, v) in expansion_obj {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                }
                
                // Recurse into nested objects
                for (_, val) in map.iter_mut() {
                    self.expand_fragments_recursive(val, fragments)?;
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.expand_fragments_recursive(item, fragments)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    // Parsing utilities

    fn parse_uuid(&self, value: Option<&Value>) -> Result<Uuid> {
        let id_str = self.parse_string(value, "id")?;
        Uuid::parse_str(&id_str)
            .map_err(|e| KnowledgeGraphError::ParseError {
                message: format!("Failed to parse UUID: {}", e),
                field: Some("id".to_string()),
                raw_data: Some(id_str),
                source_error: Some(e.to_string()),
            })
    }

    fn parse_string(&self, value: Option<&Value>, field_name: &str) -> Result<String> {
        value
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: format!("Missing or invalid field: {}", field_name),
                field: Some(field_name.to_string()),
                raw_data: value.map(|v| v.to_string()),
                source_error: None,
            })
    }

    fn parse_optional_string(&self, value: Option<&Value>) -> Option<String> {
        value.and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    fn parse_string_array(&self, value: Option<&Value>) -> Vec<String> {
        value
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_f32(&self, value: Option<&Value>, default: f32) -> Result<f32> {
        value
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .or(Some(default))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Invalid float value".to_string(),
                field: None,
                raw_data: value.map(|v| v.to_string()),
                source_error: None,
            })
    }

    fn parse_optional_f32(&self, value: Option<&Value>) -> Option<f32> {
        value.and_then(|v| v.as_f64()).map(|f| f as f32)
    }

    fn parse_f32_array(&self, value: Option<&Value>) -> Vec<f32> {
        value
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64())
                    .map(|f| f as f32)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_i32(&self, value: Option<&Value>, default: i32) -> Result<i32> {
        value
            .and_then(|v| v.as_i64())
            .map(|i| i as i32)
            .or(Some(default))
            .ok_or_else(|| KnowledgeGraphError::ParseError {
                message: "Invalid integer value".to_string(),
                field: None,
                raw_data: value.map(|v| v.to_string()),
                source_error: None,
            })
    }

    fn parse_optional_i32(&self, value: Option<&Value>) -> Option<i32> {
        value.and_then(|v| v.as_i64()).map(|i| i as i32)
    }

    fn parse_bool(&self, value: Option<&Value>, default: bool) -> Result<bool> {
        Ok(value.and_then(|v| v.as_bool()).unwrap_or(default))
    }

    fn parse_datetime(&self, value: Option<&Value>) -> Result<DateTime<Utc>> {
        let datetime_str = self.parse_string(value, "datetime")?;
        DateTime::parse_from_rfc3339(&datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|e| {
                // Try parsing as timestamp
                datetime_str.parse::<i64>()
                    .ok()
                    .and_then(|ts| DateTime::from_timestamp(ts, 0))
                    .ok_or_else(|| KnowledgeGraphError::ParseError {
                        message: format!("Invalid datetime format: {}", e),
                        field: Some("datetime".to_string()),
                        raw_data: Some(datetime_str),
                        source_error: Some(e.to_string()),
                    })
            })
    }

    fn parse_optional_datetime(&self, value: Option<&Value>) -> Option<DateTime<Utc>> {
        value
            .and_then(|v| v.as_str())
            .and_then(|s| {
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
                    .or_else(|| {
                        s.parse::<i64>()
                            .ok()
                            .and_then(|ts| DateTime::from_timestamp(ts, 0))
                    })
            })
    }
}

/// Type of mutation operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MutationOperationType {
    Add,
    Update,
    Delete,
    Upsert,
    Bulk,
}

/// Information about a conflict during mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub field: String,
    pub existing_value: Option<Value>,
    pub attempted_value: Option<Value>,
    pub resolution: Option<String>,
}

/// Result of a GraphQL mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub success: bool,
    pub message: Option<String>,
    pub affected_ids: Vec<Uuid>,
    pub affected_count: usize,
    pub operation_type: MutationOperationType,
    pub data: Option<Value>,
    pub uids: Vec<String>,
    pub conflicts: Vec<ConflictInfo>,
}

impl Default for DgraphResponseParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_concept() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "concept": {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "name": "Rust Programming",
                    "difficulty": "intermediate",
                    "category": "Programming",
                    "qualityScore": 0.85,
                    "tags": ["rust", "systems", "programming"],
                    "createdAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:00Z",
                    "version": 1
                }
            }
        });

        let result = parser.parse_concept_from_result(response);
        assert!(result.is_ok());
        
        let concept = result.unwrap();
        assert_eq!(concept.name, "Rust Programming");
        assert_eq!(concept.difficulty, "intermediate");
        assert_eq!(concept.tags.len(), 3);
    }

    #[test]
    fn test_parse_search_results() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "concepts": [
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440001",
                        "name": "Concept 1",
                        "difficulty": "beginner",
                        "category": "Test",
                        "qualityScore": 0.7,
                        "tags": [],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z",
                        "version": 1
                    },
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440002",
                        "name": "Concept 2",
                        "difficulty": "intermediate",
                        "category": "Test",
                        "qualityScore": 0.8,
                        "tags": ["test"],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z",
                        "version": 1
                    }
                ]
            }
        });

        let result = parser.parse_concepts_from_search_result(response, None);
        assert!(result.is_ok());
        
        let concepts = result.unwrap();
        assert_eq!(concepts.len(), 2);
        assert_eq!(concepts[0].name, "Concept 1");
        assert_eq!(concepts[1].name, "Concept 2");
    }

    #[test]
    fn test_parse_with_aliases() {
        let parser = DgraphResponseParser::new();
        let mut response = json!({
            "data": {
                "fromConcept:concept": {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "name": "From Concept"
                },
                "toConcept:concept": {
                    "id": "550e8400-e29b-41d4-a716-446655440001",
                    "name": "To Concept"
                }
            }
        });

        let result = parser.resolve_aliases(&mut response);
        assert!(result.is_ok());
        
        let data = response.get("data").unwrap();
        assert!(data.get("fromConcept").is_some());
        assert!(data.get("toConcept").is_some());
    }

    #[test]
    fn test_handle_graphql_errors() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "errors": [
                {
                    "message": "Variable $conceptId is not defined",
                    "extensions": {
                        "code": "GRAPHQL_VALIDATION_FAILED"
                    }
                }
            ]
        });

        let result = parser.parse_concept_from_result(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Variable $conceptId is not defined"));
    }

    #[test]
    fn test_parse_nested_relationships() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "concept": {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "name": "Main Concept",
                    "difficulty": "intermediate",
                    "category": "Test",
                    "qualityScore": 0.8,
                    "tags": [],
                    "createdAt": "2024-01-01T00:00:00Z",
                    "updatedAt": "2024-01-01T00:00:00Z",
                    "version": 1,
                    "prerequisites": [
                        {
                            "id": "550e8400-e29b-41d4-a716-446655440001",
                            "name": "Prerequisite 1",
                            "difficulty": "beginner",
                            "category": "Test",
                            "qualityScore": 0.7,
                            "tags": [],
                            "createdAt": "2024-01-01T00:00:00Z",
                            "updatedAt": "2024-01-01T00:00:00Z",
                            "version": 1
                        }
                    ]
                }
            }
        });

        let result = parser.parse_concepts_from_graph_result(response);
        assert!(result.is_ok());
        
        let concepts = result.unwrap();
        assert_eq!(concepts.len(), 2);
        assert_eq!(concepts[0].name, "Main Concept");
        assert_eq!(concepts[1].name, "Prerequisite 1");
    }

    #[test]
    fn test_parse_add_mutation_result() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "addConcept": {
                    "concept": {
                        "id": "550e8400-e29b-41d4-a716-446655440000",
                        "name": "New Concept",
                        "difficulty": "beginner",
                        "category": "Test",
                        "qualityScore": 0.8,
                        "tags": ["test"],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z",
                        "version": 1
                    },
                    "numUids": 1,
                    "uid": "0x123"
                }
            }
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_ok());
        
        let mutation_result = result.unwrap();
        assert!(mutation_result.success);
        assert_eq!(mutation_result.operation_type, MutationOperationType::Add);
        assert_eq!(mutation_result.affected_count, 1);
        assert_eq!(mutation_result.uids, vec!["0x123"]);
        assert!(mutation_result.message.unwrap().contains("created successfully"));
    }

    #[test]
    fn test_parse_update_mutation_result() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "updateConcept": {
                    "concept": {
                        "id": "550e8400-e29b-41d4-a716-446655440000",
                        "name": "Updated Concept",
                        "difficulty": "intermediate",
                        "category": "Test",
                        "qualityScore": 0.9,
                        "tags": ["updated"],
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-02T00:00:00Z",
                        "version": 2
                    },
                    "conflicts": [
                        {
                            "field": "version",
                            "existingValue": 1,
                            "attemptedValue": 2,
                            "resolution": "overwritten"
                        }
                    ]
                }
            }
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_ok());
        
        let mutation_result = result.unwrap();
        assert!(mutation_result.success);
        assert_eq!(mutation_result.operation_type, MutationOperationType::Update);
        assert_eq!(mutation_result.conflicts.len(), 1);
        assert_eq!(mutation_result.conflicts[0].field, "version");
    }

    #[test]
    fn test_parse_delete_mutation_result() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "deleteConcept": {
                    "msg": "Successfully deleted concept",
                    "numUids": 1,
                    "deletedUids": ["0x123"],
                    "deletedIds": ["550e8400-e29b-41d4-a716-446655440000"]
                }
            }
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_ok());
        
        let mutation_result = result.unwrap();
        assert!(mutation_result.success);
        assert_eq!(mutation_result.operation_type, MutationOperationType::Delete);
        assert_eq!(mutation_result.affected_count, 1);
        assert_eq!(mutation_result.uids, vec!["0x123"]);
        assert_eq!(mutation_result.affected_ids.len(), 1);
    }

    #[test]
    fn test_parse_bulk_add_mutation_result() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "addConcepts": {
                    "concepts": [
                        {
                            "id": "550e8400-e29b-41d4-a716-446655440001",
                            "name": "Concept 1",
                            "difficulty": "beginner",
                            "category": "Test",
                            "qualityScore": 0.7,
                            "tags": [],
                            "createdAt": "2024-01-01T00:00:00Z",
                            "updatedAt": "2024-01-01T00:00:00Z",
                            "version": 1
                        },
                        {
                            "id": "550e8400-e29b-41d4-a716-446655440002",
                            "name": "Concept 2",
                            "difficulty": "intermediate",
                            "category": "Test",
                            "qualityScore": 0.8,
                            "tags": [],
                            "createdAt": "2024-01-01T00:00:00Z",
                            "updatedAt": "2024-01-01T00:00:00Z",
                            "version": 1
                        }
                    ],
                    "numUids": 2
                }
            }
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_ok());
        
        let mutation_result = result.unwrap();
        assert!(mutation_result.success);
        assert_eq!(mutation_result.operation_type, MutationOperationType::Bulk);
        assert_eq!(mutation_result.affected_count, 2);
        assert_eq!(mutation_result.affected_ids.len(), 2);
        assert!(mutation_result.message.unwrap().contains("Created 2 concepts"));
    }

    #[test]
    fn test_parse_raw_mutation_result() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "mutate": {
                    "code": "Success",
                    "message": "Mutation applied successfully",
                    "uids": {
                        "concept1": "0x123",
                        "concept2": "0x124"
                    },
                    "queries": {
                        "q": [{
                            "count": 2
                        }]
                    }
                }
            }
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_ok());
        
        let mutation_result = result.unwrap();
        assert!(mutation_result.success);
        assert_eq!(mutation_result.affected_count, 2);
        assert_eq!(mutation_result.uids.len(), 2);
        assert!(mutation_result.data.is_some());
    }

    #[test]
    fn test_parse_mutation_with_errors() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "errors": [{
                "message": "Conflict: concept already exists",
                "extensions": {
                    "code": "CONFLICT"
                }
            }]
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Conflict: concept already exists"));
    }

    #[test]
    fn test_parse_generic_mutation_result() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "someCustomMutation": {
                    "success": true,
                    "message": "Custom operation to update data completed",
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "uid": "0x125",
                    "numUids": 1
                }
            }
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_ok());
        
        let mutation_result = result.unwrap();
        assert!(mutation_result.success);
        assert_eq!(mutation_result.operation_type, MutationOperationType::Update);
        assert_eq!(mutation_result.affected_ids.len(), 1);
        assert_eq!(mutation_result.uids, vec!["0x125"]);
    }

    #[test]
    fn test_parse_learning_resource_mutation() {
        let parser = DgraphResponseParser::new();
        
        let response = json!({
            "data": {
                "addLearningResource": {
                    "resource": {
                        "id": "550e8400-e29b-41d4-a716-446655440000",
                        "url": "https://example.com/resource",
                        "title": "Test Resource",
                        "resourceType": "article",
                        "createdAt": "2024-01-01T00:00:00Z",
                        "updatedAt": "2024-01-01T00:00:00Z"
                    }
                }
            }
        });

        let result = parser.parse_mutation_result(response);
        assert!(result.is_ok());
        
        let mutation_result = result.unwrap();
        assert!(mutation_result.success);
        assert_eq!(mutation_result.operation_type, MutationOperationType::Add);
        assert!(mutation_result.message.unwrap().contains("Learning resource created"));
    }

    #[test]
    fn test_parse_conflict_info() {
        let parser = DgraphResponseParser::new();
        
        let conflict_data = json!([
            {
                "field": "name",
                "existingValue": "Old Name",
                "attemptedValue": "New Name",
                "resolution": "rejected"
            },
            {
                "field": "version",
                "existingValue": 2,
                "attemptedValue": 1,
                "resolution": "merge"
            }
        ]);

        let conflicts = parser.parse_conflict_info(Some(&conflict_data));
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].field, "name");
        assert_eq!(conflicts[1].field, "version");
        assert_eq!(conflicts[1].resolution, Some("merge".to_string()));
    }
}