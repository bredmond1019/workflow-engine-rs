/*!
# Workflow Schema Definitions

This module defines the YAML schema structures for workflows, enabling
declarative workflow definitions that can be parsed and executed.

Task 2.1: Define research_to_documentation workflow YAML schema
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main workflow definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Unique workflow name/identifier
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Workflow version for schema evolution
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Input schema for the workflow
    #[serde(default)]
    pub inputs: HashMap<String, InputDefinition>,
    
    /// Ordered list of workflow steps
    pub steps: Vec<StepDefinition>,
    
    /// Output mappings from final steps
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    
    /// Workflow-level configuration
    #[serde(default)]
    pub config: WorkflowConfig,
}

/// Individual step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDefinition {
    /// Unique step identifier
    pub id: String,
    
    /// Step name for display
    #[serde(default)]
    pub name: Option<String>,
    
    /// Type of step to execute
    #[serde(rename = "type")]
    pub step_type: StepType,
    
    /// Input data for this step
    #[serde(default)]
    pub input: serde_json::Value,
    
    /// Steps that must complete before this one
    #[serde(default)]
    pub depends_on: Vec<String>,
    
    /// Whether this step can run in parallel with others
    #[serde(default)]
    pub parallel: bool,
    
    /// Step-specific configuration
    #[serde(default)]
    pub config: StepConfig,
}

/// Types of workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StepType {
    /// Execute a local workflow node
    #[serde(rename = "node")]
    Node {
        /// Node type to instantiate
        node: String,
    },
    
    /// Make a cross-system call
    #[serde(rename = "cross_system")]
    CrossSystem {
        /// Target system identifier
        system: String,
        /// Operation/method to call
        operation: String,
        /// Optional specific agent/service
        agent: Option<String>,
    },
    
    /// Conditional logic step
    #[serde(rename = "condition")]
    Condition {
        /// Condition expression
        condition: String,
        /// Steps to execute if true
        then_steps: Vec<StepDefinition>,
        /// Steps to execute if false
        else_steps: Option<Vec<StepDefinition>>,
    },
    
    /// Loop over a collection
    #[serde(rename = "loop")]
    Loop {
        /// Collection to iterate over
        items: String,
        /// Steps to execute for each item
        steps: Vec<StepDefinition>,
    },
    
    /// Transform data using a template
    #[serde(rename = "transform")]
    Transform {
        /// Template engine to use (handlebars, jinja2, etc.)
        engine: String,
        /// Template content
        template: String,
    },
}

/// Input parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDefinition {
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Whether this parameter is required
    #[serde(default)]
    pub required: bool,
    
    /// Default value if not provided
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    
    /// Validation rules
    #[serde(default)]
    pub validation: Option<ValidationRules>,
}

/// Validation rules for inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Minimum value/length
    pub min: Option<f64>,
    
    /// Maximum value/length
    pub max: Option<f64>,
    
    /// Pattern to match (regex)
    pub pattern: Option<String>,
    
    /// Allowed values (enum)
    pub enum_values: Option<Vec<serde_json::Value>>,
}

/// Workflow-level configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowConfig {
    /// Maximum execution time in seconds
    pub timeout: Option<u64>,
    
    /// Number of retry attempts for failed steps
    pub retries: Option<u32>,
    
    /// Whether to continue on step failures
    pub continue_on_error: Option<bool>,
    
    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
    
    /// Notification settings
    pub notifications: Option<NotificationConfig>,
}

/// Step-level configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepConfig {
    /// Step timeout in seconds
    pub timeout: Option<u64>,
    
    /// Retry attempts for this step
    pub retries: Option<u32>,
    
    /// Whether to cache the result
    pub cache: Option<bool>,
    
    /// Cache TTL in seconds
    pub cache_ttl: Option<u64>,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Notify on workflow completion
    pub on_completion: Option<bool>,
    
    /// Notify on workflow failure
    pub on_failure: Option<bool>,
    
    /// Webhook URL for notifications
    pub webhook_url: Option<String>,
    
    /// Email addresses for notifications
    pub email: Option<Vec<String>>,
}

/// Runtime workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// Unique instance ID
    pub id: Uuid,
    
    /// Workflow definition being executed
    pub workflow: WorkflowDefinition,
    
    /// Current execution status
    pub status: WorkflowStatus,
    
    /// Input values provided at runtime
    pub inputs: serde_json::Value,
    
    /// Step execution states
    pub steps: HashMap<String, StepExecution>,
    
    /// Final workflow outputs
    pub outputs: Option<serde_json::Value>,
    
    /// Execution timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Error information if failed
    pub error: Option<WorkflowError>,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    /// Workflow created but not started
    Created,
    /// Currently executing
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled by user
    Cancelled,
    /// Paused awaiting input
    Paused,
}

/// Individual step execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    /// Step status
    pub status: StepStatus,
    
    /// Step input data
    pub input: serde_json::Value,
    
    /// Step output data
    pub output: Option<serde_json::Value>,
    
    /// Execution timestamps
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Error if step failed
    pub error: Option<String>,
    
    /// Retry attempt number
    pub attempt: u32,
}

/// Step execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    /// Waiting for dependencies
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Skipped due to conditions
    Skipped,
}

/// Workflow execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    /// Error message
    pub message: String,
    
    /// Error code
    pub code: String,
    
    /// Step that caused the error
    pub step_id: Option<String>,
    
    /// Detailed error information
    pub details: Option<serde_json::Value>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Predefined workflow templates
pub mod templates {
    use super::*;
    
    /// Research to Documentation workflow template
    pub fn research_to_documentation() -> WorkflowDefinition {
        WorkflowDefinition {
            name: "research_to_documentation".to_string(),
            description: "Research a topic and create comprehensive documentation".to_string(),
            version: "1.0".to_string(),
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("topic".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "The topic to research".to_string(),
                    required: true,
                    default: None,
                    validation: Some(ValidationRules {
                        min: Some(3.0),
                        max: Some(200.0),
                        pattern: None,
                        enum_values: None,
                    }),
                });
                inputs.insert("difficulty".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "Difficulty level for the research".to_string(),
                    required: false,
                    default: Some(serde_json::Value::String("intermediate".to_string())),
                    validation: Some(ValidationRules {
                        min: None,
                        max: None,
                        pattern: None,
                        enum_values: Some(vec![
                            serde_json::Value::String("beginner".to_string()),
                            serde_json::Value::String("intermediate".to_string()),
                            serde_json::Value::String("advanced".to_string()),
                        ]),
                    }),
                });
                inputs.insert("max_sources".to_string(), InputDefinition {
                    param_type: "integer".to_string(),
                    description: "Maximum number of sources to research".to_string(),
                    required: false,
                    default: Some(serde_json::Value::Number(serde_json::Number::from(10))),
                    validation: Some(ValidationRules {
                        min: Some(1.0),
                        max: Some(50.0),
                        pattern: None,
                        enum_values: None,
                    }),
                });
                inputs
            },
            steps: vec![
                StepDefinition {
                    id: "research_topic".to_string(),
                    name: Some("Research Topic".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "research_workflow".to_string(),
                        agent: Some("orchestrator".to_string()),
                    },
                    input: serde_json::json!({
                        "topic": "{{ input.topic }}",
                        "difficulty": "{{ input.difficulty | default('intermediate') }}",
                        "max_sources": "{{ input.max_sources | default(10) }}",
                        "context": {
                            "source": "workflow_system",
                            "workflow": "research_to_documentation"
                        }
                    }),
                    depends_on: vec![],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(300), // 5 minutes
                        retries: Some(2),
                        cache: Some(true),
                        cache_ttl: Some(3600), // 1 hour
                    },
                },
                StepDefinition {
                    id: "create_notion_page".to_string(),
                    name: Some("Create Documentation Page".to_string()),
                    step_type: StepType::Node {
                        node: "NotionClientNode".to_string(),
                    },
                    input: serde_json::json!({
                        "title": "Research: {{ input.topic }}",
                        "content": {
                            "type": "template",
                            "template": "notion_research_page",
                            "data": {
                                "topic": "{{ input.topic }}",
                                "summary": "{{ steps.research_topic.output.summary }}",
                                "key_points": "{{ steps.research_topic.output.key_points }}",
                                "sources": "{{ steps.research_topic.output.sources }}",
                                "research_data": "{{ steps.research_topic.output }}"
                            }
                        },
                        "parent_id": "{{ env.NOTION_RESEARCH_FOLDER }}",
                        "properties": {
                            "Topic": "{{ input.topic }}",
                            "Difficulty": "{{ input.difficulty }}",
                            "Source Count": "{{ steps.research_topic.output.sources | length }}",
                            "Created": "{{ now() }}"
                        }
                    }),
                    depends_on: vec!["research_topic".to_string()],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(60),
                        retries: Some(3),
                        cache: Some(false),
                        cache_ttl: None,
                    },
                },
            ],
            outputs: {
                let mut outputs = HashMap::new();
                outputs.insert("research_summary".to_string(), "{{ steps.research_topic.output.summary }}".to_string());
                outputs.insert("notion_page_url".to_string(), "{{ steps.create_notion_page.output.url }}".to_string());
                outputs.insert("notion_page_id".to_string(), "{{ steps.create_notion_page.output.page_id }}".to_string());
                outputs.insert("source_count".to_string(), "{{ steps.research_topic.output.sources | length }}".to_string());
                outputs
            },
            config: WorkflowConfig {
                timeout: Some(600), // 10 minutes total
                retries: Some(1),
                continue_on_error: Some(false),
                environment: {
                    let mut env = HashMap::new();
                    env.insert("NOTION_RESEARCH_FOLDER".to_string(), "${NOTION_RESEARCH_FOLDER}".to_string());
                    env
                },
                notifications: Some(NotificationConfig {
                    on_completion: Some(true),
                    on_failure: Some(true),
                    webhook_url: None,
                    email: None,
                }),
            },
        }
    }
    
    /// Research to Slack workflow template
    pub fn research_to_slack() -> WorkflowDefinition {
        WorkflowDefinition {
            name: "research_to_slack".to_string(),
            description: "Research a topic and post summary to Slack".to_string(),
            version: "1.0".to_string(),
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("topic".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "The topic to research".to_string(),
                    required: true,
                    default: None,
                    validation: None,
                });
                inputs.insert("channel".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "Slack channel to post to".to_string(),
                    required: true,
                    default: None,
                    validation: None,
                });
                inputs
            },
            steps: vec![
                StepDefinition {
                    id: "research_topic".to_string(),
                    name: Some("Research Topic".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "research_workflow".to_string(),
                        agent: Some("orchestrator".to_string()),
                    },
                    input: serde_json::json!({
                        "topic": "{{ input.topic }}",
                        "difficulty": "intermediate",
                        "max_sources": 5
                    }),
                    depends_on: vec![],
                    parallel: false,
                    config: StepConfig::default(),
                },
                StepDefinition {
                    id: "post_to_slack".to_string(),
                    name: Some("Post to Slack".to_string()),
                    step_type: StepType::Node {
                        node: "SlackClientNode".to_string(),
                    },
                    input: serde_json::json!({
                        "channel": "{{ input.channel }}",
                        "message": {
                            "type": "template",
                            "template": "slack_research_summary",
                            "data": {
                                "topic": "{{ input.topic }}",
                                "summary": "{{ steps.research_topic.output.summary }}",
                                "key_points": "{{ steps.research_topic.output.key_points }}"
                            }
                        }
                    }),
                    depends_on: vec!["research_topic".to_string()],
                    parallel: false,
                    config: StepConfig::default(),
                },
            ],
            outputs: {
                let mut outputs = HashMap::new();
                outputs.insert("slack_message_id".to_string(), "{{ steps.post_to_slack.output.message_id }}".to_string());
                outputs
            },
            config: WorkflowConfig::default(),
        }
    }
    
    /// User Query Processing workflow template
    pub fn user_query_processing() -> WorkflowDefinition {
        WorkflowDefinition {
            name: "user_query_processing".to_string(),
            description: "Process user queries through research, analysis, and response generation".to_string(),
            version: "1.0".to_string(),
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("query".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "User's query or question".to_string(),
                    required: true,
                    default: None,
                    validation: Some(ValidationRules {
                        min: Some(5.0),
                        max: Some(1000.0),
                        pattern: None,
                        enum_values: None,
                    }),
                });
                inputs.insert("context".to_string(), InputDefinition {
                    param_type: "object".to_string(),
                    description: "Additional context for the query".to_string(),
                    required: false,
                    default: Some(serde_json::json!({})),
                    validation: None,
                });
                inputs.insert("response_format".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "Desired response format".to_string(),
                    required: false,
                    default: Some(serde_json::Value::String("comprehensive".to_string())),
                    validation: Some(ValidationRules {
                        min: None,
                        max: None,
                        pattern: None,
                        enum_values: Some(vec![
                            serde_json::Value::String("brief".to_string()),
                            serde_json::Value::String("comprehensive".to_string()),
                            serde_json::Value::String("technical".to_string()),
                            serde_json::Value::String("educational".to_string()),
                        ]),
                    }),
                });
                inputs
            },
            steps: vec![
                StepDefinition {
                    id: "analyze_query".to_string(),
                    name: Some("Analyze User Query".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "analyze_query".to_string(),
                        agent: Some("query_analyzer".to_string()),
                    },
                    input: serde_json::json!({
                        "query": "{{ input.query }}",
                        "context": "{{ input.context }}",
                        "analyze_intent": true,
                        "extract_keywords": true,
                        "determine_complexity": true
                    }),
                    depends_on: vec![],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(30),
                        retries: Some(2),
                        cache: Some(true),
                        cache_ttl: Some(300), // 5 minutes
                    },
                },
                StepDefinition {
                    id: "search_knowledge_base".to_string(),
                    name: Some("Search Knowledge Base".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "search_knowledge".to_string(),
                        agent: Some("knowledge_searcher".to_string()),
                    },
                    input: serde_json::json!({
                        "keywords": "{{ steps.analyze_query.output.keywords }}",
                        "intent": "{{ steps.analyze_query.output.intent }}",
                        "search_scope": ["notion", "slack", "helpscout"],
                        "max_results": 10
                    }),
                    depends_on: vec!["analyze_query".to_string()],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(60),
                        retries: Some(2),
                        cache: Some(true),
                        cache_ttl: Some(600), // 10 minutes
                    },
                },
                StepDefinition {
                    id: "generate_response".to_string(),
                    name: Some("Generate Response".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "generate_response".to_string(),
                        agent: Some("response_generator".to_string()),
                    },
                    input: serde_json::json!({
                        "original_query": "{{ input.query }}",
                        "query_analysis": "{{ steps.analyze_query.output }}",
                        "knowledge_results": "{{ steps.search_knowledge_base.output }}",
                        "response_format": "{{ input.response_format }}",
                        "context": "{{ input.context }}"
                    }),
                    depends_on: vec!["analyze_query".to_string(), "search_knowledge_base".to_string()],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(120),
                        retries: Some(2),
                        cache: Some(false),
                        cache_ttl: None,
                    },
                },
                StepDefinition {
                    id: "log_interaction".to_string(),
                    name: Some("Log User Interaction".to_string()),
                    step_type: StepType::Node {
                        node: "InteractionLogger".to_string(),
                    },
                    input: serde_json::json!({
                        "query": "{{ input.query }}",
                        "response": "{{ steps.generate_response.output.response }}",
                        "confidence": "{{ steps.generate_response.output.confidence }}",
                        "sources_used": "{{ steps.search_knowledge_base.output.sources }}",
                        "timestamp": "{{ now }}"
                    }),
                    depends_on: vec!["generate_response".to_string()],
                    parallel: true,
                    config: StepConfig {
                        timeout: Some(15),
                        retries: Some(1),
                        cache: Some(false),
                        cache_ttl: None,
                    },
                },
            ],
            outputs: {
                let mut outputs = HashMap::new();
                outputs.insert("response".to_string(), "{{ steps.generate_response.output.response }}".to_string());
                outputs.insert("confidence".to_string(), "{{ steps.generate_response.output.confidence }}".to_string());
                outputs.insert("sources".to_string(), "{{ steps.search_knowledge_base.output.sources }}".to_string());
                outputs.insert("intent".to_string(), "{{ steps.analyze_query.output.intent }}".to_string());
                outputs.insert("keywords".to_string(), "{{ steps.analyze_query.output.keywords }}".to_string());
                outputs
            },
            config: WorkflowConfig {
                timeout: Some(300), // 5 minutes total
                retries: Some(1),
                continue_on_error: Some(false),
                environment: HashMap::new(),
                notifications: Some(NotificationConfig {
                    on_completion: Some(false),
                    on_failure: Some(true),
                    webhook_url: None,
                    email: None,
                }),
            },
        }
    }
    
    /// AI Content Generation workflow template
    pub fn ai_content_generation() -> WorkflowDefinition {
        WorkflowDefinition {
            name: "ai_content_generation".to_string(),
            description: "Generate AI-powered content based on requirements and templates".to_string(),
            version: "1.0".to_string(),
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("content_type".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "Type of content to generate".to_string(),
                    required: true,
                    default: None,
                    validation: Some(ValidationRules {
                        min: None,
                        max: None,
                        pattern: None,
                        enum_values: Some(vec![
                            serde_json::Value::String("blog_post".to_string()),
                            serde_json::Value::String("documentation".to_string()),
                            serde_json::Value::String("tutorial".to_string()),
                            serde_json::Value::String("api_docs".to_string()),
                            serde_json::Value::String("user_guide".to_string()),
                            serde_json::Value::String("faq".to_string()),
                        ]),
                    }),
                });
                inputs.insert("topic".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "Main topic or subject for the content".to_string(),
                    required: true,
                    default: None,
                    validation: Some(ValidationRules {
                        min: Some(3.0),
                        max: Some(200.0),
                        pattern: None,
                        enum_values: None,
                    }),
                });
                inputs.insert("requirements".to_string(), InputDefinition {
                    param_type: "object".to_string(),
                    description: "Specific requirements and constraints".to_string(),
                    required: false,
                    default: Some(serde_json::json!({
                        "length": "medium",
                        "audience": "general",
                        "tone": "professional"
                    })),
                    validation: None,
                });
                inputs.insert("template_id".to_string(), InputDefinition {
                    param_type: "string".to_string(),
                    description: "Optional template to use for content structure".to_string(),
                    required: false,
                    default: None,
                    validation: None,
                });
                inputs
            },
            steps: vec![
                StepDefinition {
                    id: "research_content".to_string(),
                    name: Some("Research Content Topic".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "research_workflow".to_string(),
                        agent: Some("content_researcher".to_string()),
                    },
                    input: serde_json::json!({
                        "topic": "{{ input.topic }}",
                        "content_type": "{{ input.content_type }}",
                        "research_depth": "{{ input.requirements.research_depth | default('medium') }}",
                        "max_sources": "{{ input.requirements.max_sources | default(8) }}",
                        "focus_areas": "{{ input.requirements.focus_areas | default([]) }}"
                    }),
                    depends_on: vec![],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(180), // 3 minutes
                        retries: Some(2),
                        cache: Some(true),
                        cache_ttl: Some(1800), // 30 minutes
                    },
                },
                StepDefinition {
                    id: "generate_outline".to_string(),
                    name: Some("Generate Content Outline".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "generate_outline".to_string(),
                        agent: Some("content_planner".to_string()),
                    },
                    input: serde_json::json!({
                        "topic": "{{ input.topic }}",
                        "content_type": "{{ input.content_type }}",
                        "research_data": "{{ steps.research_content.output }}",
                        "requirements": "{{ input.requirements }}",
                        "template_id": "{{ input.template_id }}"
                    }),
                    depends_on: vec!["research_content".to_string()],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(60),
                        retries: Some(2),
                        cache: Some(true),
                        cache_ttl: Some(900), // 15 minutes
                    },
                },
                StepDefinition {
                    id: "generate_content".to_string(),
                    name: Some("Generate Full Content".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "generate_content".to_string(),
                        agent: Some("content_writer".to_string()),
                    },
                    input: serde_json::json!({
                        "outline": "{{ steps.generate_outline.output }}",
                        "research_data": "{{ steps.research_content.output }}",
                        "requirements": "{{ input.requirements }}",
                        "topic": "{{ input.topic }}",
                        "content_type": "{{ input.content_type }}"
                    }),
                    depends_on: vec!["generate_outline".to_string()],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(300), // 5 minutes
                        retries: Some(2),
                        cache: Some(false),
                        cache_ttl: None,
                    },
                },
                StepDefinition {
                    id: "review_content".to_string(),
                    name: Some("Review and Improve Content".to_string()),
                    step_type: StepType::CrossSystem {
                        system: "ai-tutor".to_string(),
                        operation: "review_content".to_string(),
                        agent: Some("content_reviewer".to_string()),
                    },
                    input: serde_json::json!({
                        "content": "{{ steps.generate_content.output }}",
                        "requirements": "{{ input.requirements }}",
                        "topic": "{{ input.topic }}",
                        "content_type": "{{ input.content_type }}",
                        "check_quality": true,
                        "check_accuracy": true,
                        "check_completeness": true
                    }),
                    depends_on: vec!["generate_content".to_string()],
                    parallel: false,
                    config: StepConfig {
                        timeout: Some(120),
                        retries: Some(1),
                        cache: Some(false),
                        cache_ttl: None,
                    },
                },
                StepDefinition {
                    id: "save_to_notion".to_string(),
                    name: Some("Save Content to Notion".to_string()),
                    step_type: StepType::Node {
                        node: "NotionClientNode".to_string(),
                    },
                    input: serde_json::json!({
                        "title": "{{ input.content_type | title }}: {{ input.topic }}",
                        "content": "{{ steps.review_content.output.final_content }}",
                        "parent_id": "{{ env.NOTION_CONTENT_FOLDER }}",
                        "properties": {
                            "Content Type": "{{ input.content_type }}",
                            "Topic": "{{ input.topic }}",
                            "Status": "Generated",
                            "Created": "{{ now }}",
                            "Word Count": "{{ steps.review_content.output.word_count }}",
                            "Quality Score": "{{ steps.review_content.output.quality_score }}"
                        }
                    }),
                    depends_on: vec!["review_content".to_string()],
                    parallel: true,
                    config: StepConfig {
                        timeout: Some(60),
                        retries: Some(3),
                        cache: Some(false),
                        cache_ttl: None,
                    },
                },
            ],
            outputs: {
                let mut outputs = HashMap::new();
                outputs.insert("content".to_string(), "{{ steps.review_content.output.final_content }}".to_string());
                outputs.insert("outline".to_string(), "{{ steps.generate_outline.output }}".to_string());
                outputs.insert("quality_score".to_string(), "{{ steps.review_content.output.quality_score }}".to_string());
                outputs.insert("word_count".to_string(), "{{ steps.review_content.output.word_count }}".to_string());
                outputs.insert("notion_page_url".to_string(), "{{ steps.save_to_notion.output.url }}".to_string());
                outputs.insert("sources_count".to_string(), "{{ steps.research_content.output.sources | length }}".to_string());
                outputs
            },
            config: WorkflowConfig {
                timeout: Some(900), // 15 minutes total
                retries: Some(1),
                continue_on_error: Some(false),
                environment: {
                    let mut env = HashMap::new();
                    env.insert("NOTION_CONTENT_FOLDER".to_string(), "${NOTION_CONTENT_FOLDER}".to_string());
                    env
                },
                notifications: Some(NotificationConfig {
                    on_completion: Some(true),
                    on_failure: Some(true),
                    webhook_url: None,
                    email: None,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_definition_serialization() {
        let workflow = templates::research_to_documentation();
        
        // Test serialization to YAML
        let yaml = serde_yaml::to_string(&workflow).unwrap();
        assert!(yaml.contains("research_to_documentation"));
        assert!(yaml.contains("cross_system"));
        
        // Test deserialization from YAML
        let deserialized: WorkflowDefinition = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.name, workflow.name);
        assert_eq!(deserialized.steps.len(), workflow.steps.len());
    }
    
    #[test]
    fn test_step_type_variants() {
        let cross_system_step = StepType::CrossSystem {
            system: "ai-tutor".to_string(),
            operation: "research".to_string(),
            agent: None,
        };
        
        let node_step = StepType::Node {
            node: "NotionClient".to_string(),
        };
        
        // Test serialization
        let yaml1 = serde_yaml::to_string(&cross_system_step).unwrap();
        let yaml2 = serde_yaml::to_string(&node_step).unwrap();
        
        assert!(yaml1.contains("cross_system"));
        assert!(yaml2.contains("node"));
    }
    
    #[test]
    fn test_workflow_instance_creation() {
        let workflow = templates::research_to_documentation();
        let instance = WorkflowInstance {
            id: Uuid::new_v4(),
            workflow: workflow.clone(),
            status: WorkflowStatus::Created,
            inputs: serde_json::json!({"topic": "machine learning"}),
            steps: HashMap::new(),
            outputs: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        };
        
        assert_eq!(instance.status, WorkflowStatus::Created);
        assert_eq!(instance.workflow.name, "research_to_documentation");
    }
}