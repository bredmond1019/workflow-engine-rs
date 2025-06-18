//! Query building and execution

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    ShortestPath,
    FindSimilar,
    GetPrerequisites,
    SearchConcepts,
    GetConcept,
    CreateConcept,
    UpdateConcept,
    DeleteConcept,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParameters {
    pub from_concept: Option<String>,
    pub to_concept: Option<String>,
    pub concept_id: Option<String>,
    pub concept_name: Option<String>,
    pub search_term: Option<String>,
    pub max_depth: Option<u32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub constraints: Option<QueryConstraints>,
    pub concept_data: Option<serde_json::Value>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConstraints {
    pub difficulty: Vec<String>,
    pub min_quality: f32,
    pub categories: Option<Vec<String>>,
    pub include_subtopics: Option<bool>,
}

pub struct QueryBuilder;

impl QueryBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build_query(&self, query_type: QueryType, params: QueryParameters) -> Result<String> {
        match query_type {
            QueryType::ShortestPath => self.build_shortest_path_query(params),
            QueryType::FindSimilar => self.build_find_similar_query(params),
            QueryType::GetPrerequisites => self.build_prerequisites_query(params),
            QueryType::SearchConcepts => self.build_search_concepts_query(params),
            QueryType::GetConcept => self.build_get_concept_query(params),
            QueryType::CreateConcept => self.build_create_concept_mutation(params),
            QueryType::UpdateConcept => self.build_update_concept_mutation(params),
            QueryType::DeleteConcept => self.build_delete_concept_mutation(params),
        }
    }

    fn build_shortest_path_query(&self, params: QueryParameters) -> Result<String> {
        let from_concept = params.from_concept
            .ok_or_else(|| anyhow!("from_concept is required for shortest path query"))?;
        let to_concept = params.to_concept
            .ok_or_else(|| anyhow!("to_concept is required for shortest path query"))?;
        let max_depth = params.max_depth.unwrap_or(10);

        let query = format!(r#"
            query ShortestPath($from: String!, $to: String!) {{
                fromConcept: queryConcept(filter: {{ name: {{ eq: $from }} }}) {{
                    id
                    name
                    prerequisites(filter: {{ name: {{ eq: $to }} }}, first: {}) {{
                        id
                        name
                        difficulty
                        prerequisites {{
                            id
                            name
                        }}
                    }}
                }}
                toConcept: queryConcept(filter: {{ name: {{ eq: $to }} }}) {{
                    id
                    name
                }}
            }}
        "#, max_depth);

        Ok(query)
    }

    fn build_find_similar_query(&self, params: QueryParameters) -> Result<String> {
        let concept_name = params.concept_name
            .ok_or_else(|| anyhow!("concept_name is required for find similar query"))?;
        let limit = params.limit.unwrap_or(10);

        let mut filter_conditions = Vec::new();
        
        if let Some(constraints) = &params.constraints {
            if !constraints.difficulty.is_empty() {
                let difficulties = constraints.difficulty.iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(", ");
                filter_conditions.push(format!("difficulty: {{ in: [{}] }}", difficulties));
            }
            
            if constraints.min_quality > 0.0 {
                filter_conditions.push(format!("qualityScore: {{ ge: {} }}", constraints.min_quality));
            }

            if let Some(categories) = &constraints.categories {
                if !categories.is_empty() {
                    let cats = categories.iter()
                        .map(|c| format!("\"{}\"", c))
                        .collect::<Vec<_>>()
                        .join(", ");
                    filter_conditions.push(format!("category: {{ in: [{}] }}", cats));
                }
            }
        }

        let filter_str = if !filter_conditions.is_empty() {
            format!("filter: {{ {} }}, ", filter_conditions.join(", "))
        } else {
            String::new()
        };

        let query = format!(r#"
            query FindSimilar($conceptName: String!) {{
                baseConcept: queryConcept(filter: {{ name: {{ eq: $conceptName }} }}) {{
                    id
                    name
                    category
                    tags
                    relatedTo({}first: {}) {{
                        id
                        name
                        difficulty
                        category
                        qualityScore
                        estimatedTime
                    }}
                }}
            }}
        "#, filter_str, limit);

        Ok(query)
    }

    fn build_prerequisites_query(&self, params: QueryParameters) -> Result<String> {
        let concept_name = params.concept_name
            .ok_or_else(|| anyhow!("concept_name is required for prerequisites query"))?;
        let max_depth = params.max_depth.unwrap_or(5);

        let query = format!(r#"
            query GetPrerequisites($conceptName: String!) {{
                concept: queryConcept(filter: {{ name: {{ eq: $conceptName }} }}) {{
                    id
                    name
                    difficulty
                    prerequisites(first: {}) {{
                        id
                        name
                        difficulty
                        category
                        estimatedTime
                        prerequisites {{
                            id
                            name
                            difficulty
                        }}
                    }}
                }}
            }}
        "#, max_depth);

        Ok(query)
    }

    fn build_search_concepts_query(&self, params: QueryParameters) -> Result<String> {
        let search_term = params.search_term
            .ok_or_else(|| anyhow!("search_term is required for search concepts query"))?;
        let limit = params.limit.unwrap_or(20);
        let offset = params.offset.unwrap_or(0);

        let mut filter_conditions = vec![
            format!("name: {{ anyoftext: \"{}\" }}", search_term)
        ];

        if let Some(constraints) = &params.constraints {
            if !constraints.difficulty.is_empty() {
                let difficulties = constraints.difficulty.iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(", ");
                filter_conditions.push(format!("difficulty: {{ in: [{}] }}", difficulties));
            }
            
            if constraints.min_quality > 0.0 {
                filter_conditions.push(format!("qualityScore: {{ ge: {} }}", constraints.min_quality));
            }

            if let Some(categories) = &constraints.categories {
                if !categories.is_empty() {
                    let cats = categories.iter()
                        .map(|c| format!("\"{}\"", c))
                        .collect::<Vec<_>>()
                        .join(", ");
                    filter_conditions.push(format!("category: {{ in: [{}] }}", cats));
                }
            }
        }

        // Build sorting clause
        let sort_clause = if let Some(sort_by) = &params.sort_by {
            let order = params.sort_order.as_deref().unwrap_or("asc");
            match sort_by.as_str() {
                "name" => format!("order: {{ {}: {} }}", sort_by, order),
                "qualityScore" => format!("order: {{ {}: {} }}", sort_by, order),
                "estimatedTime" => format!("order: {{ {}: {} }}", sort_by, order),
                "createdAt" => format!("order: {{ {}: {} }}", sort_by, order),
                "updatedAt" => format!("order: {{ {}: {} }}", sort_by, order),
                _ => "order: { name: asc }".to_string(), // Default sorting
            }
        } else {
            "order: { qualityScore: desc }".to_string() // Default to quality score descending
        };

        let query = format!(r#"
            query SearchConcepts($searchTerm: String!) {{
                concepts: queryConcept(
                    filter: {{ {} }},
                    {},
                    first: {},
                    offset: {}
                ) {{
                    id
                    name
                    description
                    difficulty
                    category
                    subcategory
                    tags
                    qualityScore
                    estimatedTime
                    prerequisites {{
                        id
                        name
                    }}
                    resources {{
                        id
                        title
                        resourceType
                        url
                    }}
                    createdAt
                    updatedAt
                }}
                conceptsAggregate: aggregateConcept(
                    filter: {{ {} }}
                ) {{
                    count
                }}
            }}
        "#, 
        filter_conditions.join(", "),
        sort_clause,
        limit,
        offset,
        filter_conditions.join(", ")
        );

        Ok(query)
    }

    fn build_get_concept_query(&self, params: QueryParameters) -> Result<String> {
        let concept_id = params.concept_id
            .ok_or_else(|| anyhow!("concept_id is required for get concept query"))?;

        let include_subtopics = params.constraints
            .as_ref()
            .and_then(|c| c.include_subtopics)
            .unwrap_or(false);

        let subtopics_field = if include_subtopics {
            r#"
                subtopics {
                    id
                    name
                    difficulty
                    estimatedTime
                }
            "#
        } else {
            ""
        };

        let query = format!(r#"
            query GetConcept($conceptId: ID!) {{
                concept: getConcept(id: $conceptId) {{
                    id
                    name
                    description
                    difficulty
                    category
                    subcategory
                    tags
                    qualityScore
                    estimatedTime
                    prerequisites {{
                        id
                        name
                        difficulty
                        estimatedTime
                    }}
                    enabledBy {{
                        id
                        name
                        difficulty
                    }}
                    relatedTo {{
                        id
                        name
                        difficulty
                        category
                    }}
                    resources {{
                        id
                        title
                        resourceType
                        url
                        quality
                        duration
                    }}
                    {}
                    createdAt
                    updatedAt
                    version
                }}
            }}
        "#, subtopics_field);

        Ok(query)
    }

    fn build_create_concept_mutation(&self, params: QueryParameters) -> Result<String> {
        let concept_data = params.concept_data
            .ok_or_else(|| anyhow!("concept_data is required for create concept mutation"))?;

        // Extract required fields from concept_data
        let name = concept_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("name is required in concept_data"))?;
        
        let difficulty = concept_data.get("difficulty")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("difficulty is required in concept_data"))?;
        
        let category = concept_data.get("category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("category is required in concept_data"))?;

        let description = concept_data.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let subcategory = concept_data.get("subcategory")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let tags = concept_data.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let quality_score = concept_data.get("qualityScore")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        let estimated_time = concept_data.get("estimatedTime")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let mutation = format!(r#"
            mutation CreateConcept {{
                addConcept(input: [{{
                    name: "{}"
                    description: "{}"
                    difficulty: "{}"
                    category: "{}"
                    subcategory: "{}"
                    tags: [{}]
                    qualityScore: {}
                    estimatedTime: {}
                    version: 1
                }}]) {{
                    concept {{
                        id
                        name
                        difficulty
                        category
                        qualityScore
                        createdAt
                    }}
                }}
            }}
        "#, name, description, difficulty, category, subcategory, tags, quality_score, estimated_time);

        Ok(mutation)
    }

    fn build_update_concept_mutation(&self, params: QueryParameters) -> Result<String> {
        let concept_id = params.concept_id
            .ok_or_else(|| anyhow!("concept_id is required for update concept mutation"))?;
        
        let concept_data = params.concept_data
            .ok_or_else(|| anyhow!("concept_data is required for update concept mutation"))?;

        let mut update_fields = Vec::new();

        if let Some(name) = concept_data.get("name").and_then(|v| v.as_str()) {
            update_fields.push(format!("name: \"{}\"", name));
        }
        
        if let Some(description) = concept_data.get("description").and_then(|v| v.as_str()) {
            update_fields.push(format!("description: \"{}\"", description));
        }
        
        if let Some(difficulty) = concept_data.get("difficulty").and_then(|v| v.as_str()) {
            update_fields.push(format!("difficulty: \"{}\"", difficulty));
        }
        
        if let Some(quality_score) = concept_data.get("qualityScore").and_then(|v| v.as_f64()) {
            update_fields.push(format!("qualityScore: {}", quality_score));
        }

        if update_fields.is_empty() {
            return Err(anyhow!("No valid fields provided for update"));
        }

        let mutation = format!(r#"
            mutation UpdateConcept {{
                updateConcept(input: {{
                    filter: {{ id: ["{}"] }}
                    set: {{
                        {}
                    }}
                }}) {{
                    concept {{
                        id
                        name
                        description
                        difficulty
                        qualityScore
                        updatedAt
                        version
                    }}
                }}
            }}
        "#, concept_id, update_fields.join(",\n                        "));

        Ok(mutation)
    }

    fn build_delete_concept_mutation(&self, params: QueryParameters) -> Result<String> {
        let concept_id = params.concept_id
            .ok_or_else(|| anyhow!("concept_id is required for delete concept mutation"))?;

        let mutation = format!(r#"
            mutation DeleteConcept {{
                deleteConcept(filter: {{ id: ["{}"] }}) {{
                    msg
                    numUids
                }}
            }}
        "#, concept_id);

        Ok(mutation)
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for QueryParameters {
    fn default() -> Self {
        Self {
            from_concept: None,
            to_concept: None,
            concept_id: None,
            concept_name: None,
            search_term: None,
            max_depth: None,
            limit: None,
            offset: None,
            constraints: None,
            concept_data: None,
            sort_by: None,
            sort_order: None,
        }
    }
}