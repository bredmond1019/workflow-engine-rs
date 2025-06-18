# Tutorial 3: AI-Powered Automation - Building an Automated Email Summarizer

## The Business Problem

Picture this: You're a busy professional who receives 50-100 emails daily. Important messages get buried in newsletters, notifications, and low-priority correspondence. You spend 30-45 minutes each morning just scanning through emails to identify what needs immediate attention.

What if an AI could read all your emails, extract the key points, and present you with a prioritized summary? That's exactly what we'll build in this tutorial.

## What You'll Learn

- How to create an AI-powered workflow that solves real business problems
- How to structure data processing with multiple nodes
- How to integrate AI agents for intelligent text analysis
- How to handle errors gracefully in production scenarios
- How to measure the business value of your automation

## Prerequisites

- Basic Rust knowledge
- AI Architecture system installed and running
- Database configured (PostgreSQL)

## Step 1: Understanding the Solution Architecture

Before diving into code, let's understand what we're building:

```
Email Input → Validation → AI Analysis → Priority Scoring → Summary Generation → Output
```

Each step is a "node" in our workflow:
1. **Email Validator**: Ensures incoming data is properly formatted
2. **AI Analyzer**: Uses Claude/GPT to understand email content
3. **Priority Scorer**: Assigns urgency based on content and sender
4. **Summary Generator**: Creates concise, actionable summaries
5. **Output Formatter**: Prepares the final report

## Step 2: Creating the Email Validator Node

Let's start by creating a node that validates incoming email data:

```rust
use ai_architecture_core::{
    nodes::Node,
    task::TaskContext,
    error::WorkflowError,
};
use serde_json::json;
use chrono::Utc;

#[derive(Debug)]
struct EmailValidatorNode;

impl Node for EmailValidatorNode {
    fn node_name(&self) -> String {
        "Email Validator".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Get the email data from the context
        let input: serde_json::Value = context.get_event_data()?;
        
        // Validate required fields
        let email_data = input.get("emails").and_then(|v| v.as_array())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "Expected 'emails' array in input".to_string()
            })?;

        // Check each email has required fields
        let mut valid_emails = Vec::new();
        let mut validation_errors = Vec::new();

        for (index, email) in email_data.iter().enumerate() {
            let has_from = email.get("from").is_some();
            let has_subject = email.get("subject").is_some();
            let has_body = email.get("body").is_some();
            let has_timestamp = email.get("timestamp").is_some();

            if has_from && has_subject && has_body && has_timestamp {
                valid_emails.push(email.clone());
            } else {
                validation_errors.push(format!("Email {} missing required fields", index));
            }
        }

        // Store validation results
        context.update_node("validation", json!({
            "valid_count": valid_emails.len(),
            "total_count": email_data.len(),
            "errors": validation_errors,
            "validated_emails": valid_emails,
            "validation_time": Utc::now()
        }));

        // Pro tip: Always validate data early in your workflow
        println!("Validated {} of {} emails", valid_emails.len(), email_data.len());

        Ok(context)
    }
}
```

## Step 3: Building the AI Analysis Node

Now let's create the heart of our system - the AI-powered email analyzer:

```rust
use ai_architecture_core::{
    nodes::Node,
    task::TaskContext,
    error::WorkflowError,
    ai_agents::anthropic::AnthropicAgentNode,
    nodes::agent::{AgentConfig, ModelProvider},
};

#[derive(Debug)]
struct EmailAnalyzerNode {
    ai_agent: AnthropicAgentNode,
}

impl EmailAnalyzerNode {
    fn new() -> Self {
        let config = AgentConfig {
            system_prompt: r#"You are an expert email analyst. For each email:
1. Extract the main topic and key points
2. Identify any action items or deadlines
3. Determine the urgency level (1-5, where 5 is most urgent)
4. Categorize the email type (meeting, request, notification, etc.)

Focus on being concise and actionable. Ignore pleasantries and focus on substance."#.to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-sonnet-20240229".to_string(),
            mcp_server_uri: None,
        };

        Self {
            ai_agent: AnthropicAgentNode::new(config),
        }
    }
}

impl Node for EmailAnalyzerNode {
    fn node_name(&self) -> String {
        "AI Email Analyzer".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Get validated emails from previous node
        let validation_data = context.get_node_result("validation")
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "No validation data found".to_string()
            })?;

        let emails = validation_data.get("validated_emails")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "No validated emails found".to_string()
            })?;

        // Analyze each email with AI
        let mut analyzed_emails = Vec::new();

        for email in emails {
            // Prepare the prompt for AI analysis
            let analysis_prompt = json!({
                "from": email.get("from"),
                "subject": email.get("subject"),
                "body": email.get("body"),
                "timestamp": email.get("timestamp")
            });

            // Create a temporary context for AI processing
            let mut ai_context = TaskContext::new(
                "email_analysis".to_string(),
                analysis_prompt
            );

            // Process with AI
            let analyzed_context = self.ai_agent.process(ai_context)?;
            
            // Extract AI response
            if let Some(ai_result) = analyzed_context.get_node_result("ai_response") {
                analyzed_emails.push(json!({
                    "original": email,
                    "analysis": ai_result
                }));
            }
        }

        // Store analyzed results
        context.update_node("email_analysis", json!({
            "analyzed_count": analyzed_emails.len(),
            "emails": analyzed_emails,
            "analysis_timestamp": Utc::now()
        }));

        Ok(context)
    }
}
```

## Step 4: Priority Scoring System

Let's add intelligence to prioritize emails based on multiple factors:

```rust
#[derive(Debug)]
struct PriorityScorerNode;

impl Node for PriorityScorerNode {
    fn node_name(&self) -> String {
        "Priority Scorer".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let analysis_data = context.get_node_result("email_analysis")
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "No analysis data found".to_string()
            })?;

        let analyzed_emails = analysis_data.get("emails")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new());

        let mut prioritized_emails = Vec::new();

        for email_data in analyzed_emails {
            let original = email_data.get("original").unwrap_or(&json!({}));
            let analysis = email_data.get("analysis").unwrap_or(&json!({}));

            // Calculate priority score based on multiple factors
            let mut priority_score = 0.0;

            // Factor 1: AI-determined urgency (weight: 40%)
            if let Some(urgency) = analysis.get("urgency").and_then(|v| v.as_f64()) {
                priority_score += urgency * 0.4;
            }

            // Factor 2: Sender importance (weight: 30%)
            if let Some(from) = original.get("from").and_then(|v| v.as_str()) {
                if from.contains("ceo") || from.contains("director") || from.contains("manager") {
                    priority_score += 5.0 * 0.3;
                } else if from.contains("client") || from.contains("customer") {
                    priority_score += 4.0 * 0.3;
                } else {
                    priority_score += 2.0 * 0.3;
                }
            }

            // Factor 3: Keywords in subject (weight: 20%)
            if let Some(subject) = original.get("subject").and_then(|v| v.as_str()) {
                let urgent_keywords = ["urgent", "asap", "critical", "emergency", "deadline"];
                let keyword_count = urgent_keywords.iter()
                    .filter(|&keyword| subject.to_lowercase().contains(keyword))
                    .count();
                priority_score += (keyword_count as f64).min(5.0) * 0.2;
            }

            // Factor 4: Has action items (weight: 10%)
            if let Some(has_actions) = analysis.get("has_action_items").and_then(|v| v.as_bool()) {
                if has_actions {
                    priority_score += 5.0 * 0.1;
                }
            }

            prioritized_emails.push(json!({
                "email": original,
                "analysis": analysis,
                "priority_score": priority_score,
                "priority_category": match priority_score {
                    s if s >= 4.0 => "Critical",
                    s if s >= 3.0 => "High",
                    s if s >= 2.0 => "Medium",
                    _ => "Low"
                }
            }));
        }

        // Sort by priority score (highest first)
        prioritized_emails.sort_by(|a, b| {
            let score_a = a.get("priority_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let score_b = b.get("priority_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        context.update_node("prioritization", json!({
            "prioritized_emails": prioritized_emails,
            "scoring_timestamp": Utc::now()
        }));

        Ok(context)
    }
}
```

## Step 5: Summary Generation

Now let's create the final output - a concise, actionable summary:

```rust
#[derive(Debug)]
struct SummaryGeneratorNode;

impl Node for SummaryGeneratorNode {
    fn node_name(&self) -> String {
        "Summary Generator".to_string()
    }

    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let prioritization_data = context.get_node_result("prioritization")
            .ok_or_else(|| WorkflowError::ValidationError {
                message: "No prioritization data found".to_string()
            })?;

        let prioritized_emails = prioritization_data.get("prioritized_emails")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new());

        // Generate executive summary
        let critical_count = prioritized_emails.iter()
            .filter(|e| e.get("priority_category").and_then(|v| v.as_str()) == Some("Critical"))
            .count();

        let high_count = prioritized_emails.iter()
            .filter(|e| e.get("priority_category").and_then(|v| v.as_str()) == Some("High"))
            .count();

        // Create summary sections
        let mut summary_sections = vec![];

        // Critical emails section
        if critical_count > 0 {
            let mut critical_items = vec![];
            for email in prioritized_emails.iter().take(5) { // Top 5 critical
                if email.get("priority_category").and_then(|v| v.as_str()) == Some("Critical") {
                    critical_items.push(json!({
                        "from": email.get("email").and_then(|e| e.get("from")),
                        "subject": email.get("email").and_then(|e| e.get("subject")),
                        "key_points": email.get("analysis").and_then(|a| a.get("key_points")),
                        "action_items": email.get("analysis").and_then(|a| a.get("action_items"))
                    }));
                }
            }
            summary_sections.push(json!({
                "section": "Critical Items Requiring Immediate Attention",
                "items": critical_items
            }));
        }

        // Generate final summary
        let summary = json!({
            "generated_at": Utc::now(),
            "total_emails_processed": prioritized_emails.len(),
            "priority_breakdown": {
                "critical": critical_count,
                "high": high_count,
                "medium": prioritized_emails.iter()
                    .filter(|e| e.get("priority_category").and_then(|v| v.as_str()) == Some("Medium"))
                    .count(),
                "low": prioritized_emails.iter()
                    .filter(|e| e.get("priority_category").and_then(|v| v.as_str()) == Some("Low"))
                    .count()
            },
            "summary_sections": summary_sections,
            "time_saved_estimate": format!("{} minutes", prioritized_emails.len() * 2), // ~2 min per email
            "full_prioritized_list": prioritized_emails
        });

        context.update_node("email_summary", summary);

        Ok(context)
    }
}
```

## Step 6: Building and Running the Complete Workflow

Now let's put it all together:

```rust
use ai_architecture_core::{
    workflow::builder::WorkflowBuilder,
    workflow::Workflow,
};
use serde_json::json;

fn create_email_summarizer_workflow() -> Result<Workflow, WorkflowError> {
    // Build the workflow pipeline
    let workflow = WorkflowBuilder::new("email_summarizer_workflow")
        .start_with::<EmailValidatorNode>()
        .then::<EmailAnalyzerNode>()
        .then::<PriorityScorerNode>()
        .then::<SummaryGeneratorNode>()
        .build()?;

    // Register all nodes
    workflow.register_node(EmailValidatorNode);
    workflow.register_node(EmailAnalyzerNode::new());
    workflow.register_node(PriorityScorerNode);
    workflow.register_node(SummaryGeneratorNode);

    Ok(workflow)
}

// Example usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = create_email_summarizer_workflow()?;

    // Sample email data (in production, this would come from your email service)
    let email_data = json!({
        "emails": [
            {
                "from": "ceo@company.com",
                "subject": "Urgent: Board meeting prep needed",
                "body": "Hi team, we need the Q3 reports ready for tomorrow's board meeting. Please prioritize this.",
                "timestamp": "2024-01-15T09:00:00Z"
            },
            {
                "from": "newsletter@techblog.com",
                "subject": "This week in tech",
                "body": "Check out the latest trends in AI and machine learning...",
                "timestamp": "2024-01-15T08:30:00Z"
            },
            {
                "from": "client@bigcorp.com",
                "subject": "Project deadline question",
                "body": "Can we discuss moving the deadline to next Friday? Let me know your availability.",
                "timestamp": "2024-01-15T08:45:00Z"
            }
        ]
    });

    // Run the workflow
    let result = workflow.run(email_data)?;

    // Extract and display the summary
    if let Some(summary) = result.get_node_result("email_summary") {
        println!("Email Summary Generated!");
        println!("========================");
        println!("Total emails processed: {}", 
            summary.get("total_emails_processed").and_then(|v| v.as_u64()).unwrap_or(0));
        println!("Time saved: {}", 
            summary.get("time_saved_estimate").and_then(|v| v.as_str()).unwrap_or("unknown"));
        
        // Display priority breakdown
        if let Some(breakdown) = summary.get("priority_breakdown") {
            println!("\nPriority Breakdown:");
            println!("- Critical: {}", breakdown.get("critical").and_then(|v| v.as_u64()).unwrap_or(0));
            println!("- High: {}", breakdown.get("high").and_then(|v| v.as_u64()).unwrap_or(0));
            println!("- Medium: {}", breakdown.get("medium").and_then(|v| v.as_u64()).unwrap_or(0));
            println!("- Low: {}", breakdown.get("low").and_then(|v| v.as_u64()).unwrap_or(0));
        }
    }

    Ok(())
}
```

## Production Deployment Tips

### 1. Error Handling and Retry Logic

Add resilience to your workflow:

```rust
impl Node for EmailAnalyzerNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        // Implement retry logic for AI calls
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 3;

        while retry_count < MAX_RETRIES {
            match self.ai_agent.process(context.clone()) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        return Err(WorkflowError::ProcessingError {
                            message: format!("AI analysis failed after {} retries: {}", MAX_RETRIES, e)
                        });
                    }
                    // Exponential backoff
                    std::thread::sleep(std::time::Duration::from_millis(100 * 2u64.pow(retry_count)));
                }
            }
        }
        unreachable!()
    }
}
```

### 2. Performance Monitoring

Track your automation's performance:

```rust
use std::time::Instant;

impl Node for EmailValidatorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        let start = Instant::now();
        
        // ... validation logic ...
        
        let duration = start.elapsed();
        context.set_metadata("validation_duration_ms", duration.as_millis())?;
        
        // Log performance metrics
        println!("Validation completed in {:?}", duration);
        
        Ok(context)
    }
}
```

### 3. Scalability Considerations

For high-volume email processing:

```rust
// Process emails in batches
const BATCH_SIZE: usize = 10;

for chunk in emails.chunks(BATCH_SIZE) {
    // Process batch
    let batch_results = process_email_batch(chunk)?;
    // Aggregate results
    results.extend(batch_results);
}
```

## Measuring Business Value

### Time Savings Calculation

```
Traditional approach: 50 emails × 2 minutes = 100 minutes/day
Automated approach: 5 minutes to review summary
Daily time saved: 95 minutes
Annual time saved: 95 × 260 working days = 410 hours
```

### Cost-Benefit Analysis

```
Developer time to build: ~16 hours
Break-even point: 16 hours ÷ 1.58 hours/day = ~10 days
Annual ROI: (410 hours × $50/hour) - (16 hours × $100/hour) = $18,900
```

## Customization Ideas

1. **Industry-Specific Analysis**: Customize the AI prompt for your industry
2. **Integration with Calendar**: Auto-create calendar events for deadlines
3. **Team Delegation**: Route emails to appropriate team members
4. **Sentiment Analysis**: Detect frustrated customers or urgent issues
5. **Multi-Language Support**: Process emails in different languages

## Pro Tips

1. **Start Small**: Begin with a subset of emails (e.g., from specific senders)
2. **Iterate on Prompts**: Fine-tune AI prompts based on actual results
3. **Monitor AI Costs**: Track token usage and optimize prompts for efficiency
4. **User Feedback Loop**: Let users rate summaries to improve the system
5. **Security First**: Never process sensitive emails without proper encryption

## Conclusion

You've built a powerful email automation system that:
- Saves 90+ minutes daily
- Ensures critical emails never get missed
- Provides actionable summaries
- Scales with your email volume

This same pattern can be applied to many business processes: document analysis, customer feedback processing, support ticket triage, and more. The key is identifying repetitive tasks that benefit from AI's understanding capabilities.

## Next Steps

1. Deploy this workflow to your production environment
2. Connect it to your actual email service (Gmail, Outlook, etc.)
3. Add a web interface for viewing summaries
4. Set up scheduled runs (e.g., every morning at 8 AM)
5. Expand to handle attachments and meeting invites

Remember: The best automation is one that actually gets used. Start simple, prove value, then expand!