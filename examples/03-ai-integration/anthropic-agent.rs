//! # Anthropic Claude Agent Integration Example
//!
//! This example demonstrates how to integrate Anthropic's Claude models into your workflows.
//! Claude excels at long-form content analysis, complex reasoning, and nuanced understanding.
//!
//! ## What You'll Learn
//! - Setting up Anthropic Claude agent nodes
//! - Configuring different Claude models (Opus, Sonnet, Haiku)
//! - Leveraging Claude's strengths for document analysis
//! - Handling long-form content processing
//! - Comparing Claude model capabilities
//!
//! ## Prerequisites
//! Set your Anthropic API key:
//! ```bash
//! export ANTHROPIC_API_KEY="your-anthropic-api-key"
//! ```
//!
//! ## Usage
//! ```bash
//! cargo run --bin anthropic-agent
//! ```

use workflow_engine_core::{
    prelude::*,
    nodes::agent::{AgentConfig, ModelProvider},
};
use workflow_engine_nodes::ai_agents::AnthropicAgentNode;
use serde_json::json;
use serde::{Deserialize, Serialize};

/// Document analysis request structure
#[derive(Debug, Deserialize, Serialize)]
struct DocumentAnalysisRequest {
    title: String,
    content: String,
    document_type: String, // "research_paper", "legal_document", "business_report", "creative_writing"
    analysis_goals: Vec<String>, // ["summarize", "extract_insights", "identify_themes", "assess_quality"]
    target_audience: String,
}

/// Document preprocessing node optimized for Claude's capabilities
#[derive(Debug)]
struct DocumentPreprocessorNode;

impl Node for DocumentPreprocessorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("📄 DocumentPreprocessorNode: Preparing document for Claude analysis...");
        
        let request: DocumentAnalysisRequest = context.get_event_data()?;
        
        // Validate document content
        if request.content.trim().is_empty() {
            return Err(WorkflowError::validation_error("Document content cannot be empty"));
        }
        
        if request.title.trim().is_empty() {
            return Err(WorkflowError::validation_error("Document title cannot be empty"));
        }
        
        // Calculate document metrics
        let word_count = request.content.split_whitespace().count();
        let paragraph_count = request.content.split("\n\n").count();
        let sentence_count = request.content.split('.').count().saturating_sub(1);
        
        // Create comprehensive prompt optimized for Claude
        let prompt = create_claude_optimized_prompt(&request);
        
        // Store processed document data
        context.update_node("document_analysis", json!({
            "prompt": prompt,
            "document_metadata": {
                "title": request.title,
                "type": request.document_type,
                "word_count": word_count,
                "paragraph_count": paragraph_count,
                "sentence_count": sentence_count,
                "estimated_reading_time": (word_count as f64 / 200.0).ceil() as u32,
                "analysis_goals": request.analysis_goals,
                "target_audience": request.target_audience
            },
            "original_request": request,
            "prepared_at": chrono::Utc::now().to_rfc3339()
        }));
        
        // Set the prompt for Claude
        context.set_data("prompt", prompt)?;
        
        context.set_metadata("document_word_count", word_count)?;
        context.set_metadata("document_type", &request.document_type)?;
        context.set_metadata("analysis_goals_count", request.analysis_goals.len())?;
        
        println!("   📊 Document metrics: {} words, {} paragraphs, {} sentences", 
            word_count, paragraph_count, sentence_count);
        println!("   🎯 Analysis goals: {}", request.analysis_goals.join(", "));
        println!("   ✅ Document prepared for Claude analysis");
        
        Ok(context)
    }
}

/// Creates a sophisticated prompt optimized for Claude's capabilities
fn create_claude_optimized_prompt(request: &DocumentAnalysisRequest) -> String {
    let mut prompt = String::new();
    
    // Start with clear context setting
    prompt.push_str("I need you to analyze the following document with your expert analytical capabilities. ");
    
    // Add document context
    prompt.push_str(&format!(
        "This is a {} titled '{}' intended for {}.\n\n",
        request.document_type.replace('_', " "),
        request.title,
        request.target_audience
    ));
    
    // Add specific analysis instructions based on goals
    prompt.push_str("Please provide a comprehensive analysis that includes:\n\n");
    
    for (i, goal) in request.analysis_goals.iter().enumerate() {
        match goal.as_str() {
            "summarize" => {
                prompt.push_str(&format!("{}. **Executive Summary**: Provide a clear, concise summary of the main points and conclusions (2-3 paragraphs).\n", i + 1));
            }
            "extract_insights" => {
                prompt.push_str(&format!("{}. **Key Insights**: Identify and explain the most important insights, discoveries, or findings presented in the document.\n", i + 1));
            }
            "identify_themes" => {
                prompt.push_str(&format!("{}. **Major Themes**: Analyze the recurring themes, patterns, and central concepts throughout the document.\n", i + 1));
            }
            "assess_quality" => {
                prompt.push_str(&format!("{}. **Quality Assessment**: Evaluate the document's quality, including clarity, coherence, evidence support, and overall effectiveness.\n", i + 1));
            }
            "critique" => {
                prompt.push_str(&format!("{}. **Critical Analysis**: Provide constructive critique, identifying strengths, weaknesses, and areas for improvement.\n", i + 1));
            }
            "recommendations" => {
                prompt.push_str(&format!("{}. **Recommendations**: Based on your analysis, provide specific, actionable recommendations.\n", i + 1));
            }
            _ => {
                prompt.push_str(&format!("{}. **{}**: Provide analysis related to {}.\n", i + 1, goal.replace('_', " ").to_uppercase(), goal.replace('_', " ")));
            }
        }
    }
    
    // Add document type specific instructions
    match request.document_type.as_str() {
        "research_paper" => {
            prompt.push_str("\nFor this research paper, pay special attention to:\n");
            prompt.push_str("- Methodology and research design\n");
            prompt.push_str("- Evidence quality and data interpretation\n");
            prompt.push_str("- Logical flow and argumentation\n");
            prompt.push_str("- Contribution to the field\n");
        }
        "legal_document" => {
            prompt.push_str("\nFor this legal document, focus on:\n");
            prompt.push_str("- Legal precedents and citations\n");
            prompt.push_str("- Clarity of legal language\n");
            prompt.push_str("- Potential implications and consequences\n");
            prompt.push_str("- Compliance and risk factors\n");
        }
        "business_report" => {
            prompt.push_str("\nFor this business report, emphasize:\n");
            prompt.push_str("- Strategic implications and recommendations\n");
            prompt.push_str("- Data presentation and analysis quality\n");
            prompt.push_str("- Market context and competitive positioning\n");
            prompt.push_str("- Feasibility and implementation considerations\n");
        }
        "creative_writing" => {
            prompt.push_str("\nFor this creative work, consider:\n");
            prompt.push_str("- Literary devices and stylistic elements\n");
            prompt.push_str("- Character development and narrative structure\n");
            prompt.push_str("- Thematic depth and emotional impact\n");
            prompt.push_str("- Originality and creative merit\n");
        }
        _ => {
            prompt.push_str("\nProvide a thorough, balanced analysis appropriate for this document type.\n");
        }
    }
    
    prompt.push_str("\nDocument to analyze:\n\n");
    prompt.push_str("=== DOCUMENT START ===\n");
    prompt.push_str(&format!("Title: {}\n\n", request.title));
    prompt.push_str(&request.content);
    prompt.push_str("\n=== DOCUMENT END ===\n\n");
    
    prompt.push_str("Please provide your analysis in a well-structured format with clear headings for each section. Use specific examples from the document to support your points.");
    
    prompt
}

/// Advanced response processor for Claude's detailed outputs
#[derive(Debug)]
struct ClaudeResponseProcessorNode;

impl Node for ClaudeResponseProcessorNode {
    fn process(&self, mut context: TaskContext) -> Result<TaskContext, WorkflowError> {
        println!("🧠 ClaudeResponseProcessorNode: Processing Claude's analysis...");
        
        // Get Claude's response
        let ai_response = context
            .get_node_data::<serde_json::Value>("ai_response")?
            .ok_or_else(|| WorkflowError::validation_error("Missing Claude response"))?;
        
        // Get document metadata
        let document_data = context
            .get_node_data::<serde_json::Value>("document_analysis")?
            .ok_or_else(|| WorkflowError::validation_error("Missing document analysis data"))?;
        
        let claude_response = ai_response["response"]
            .as_str()
            .unwrap_or("No response available");
        
        // Parse Claude's structured response
        let parsed_analysis = parse_claude_response(claude_response);
        
        // Extract insights and metrics
        let analysis_quality = assess_analysis_quality(claude_response);
        let reading_metrics = calculate_reading_metrics(claude_response);
        
        // Create comprehensive analysis result
        let analysis_result = json!({
            "document_info": document_data["document_metadata"],
            "analysis": {
                "raw_response": claude_response,
                "parsed_sections": parsed_analysis,
                "quality_metrics": analysis_quality,
                "reading_metrics": reading_metrics
            },
            "model_info": {
                "provider": "Anthropic",
                "model": ai_response.get("model").unwrap_or(&json!("claude-3")),
                "timestamp": ai_response.get("timestamp")
            },
            "processing_summary": {
                "response_length": claude_response.len(),
                "response_word_count": claude_response.split_whitespace().count(),
                "sections_identified": parsed_analysis.len(),
                "analysis_depth": classify_analysis_depth(claude_response),
                "processing_time_estimate": estimate_processing_time(&document_data)
            }
        });
        
        context.update_node("claude_analysis", analysis_result);
        context.set_metadata("claude_response_length", claude_response.len())?;
        context.set_metadata("analysis_sections", parsed_analysis.len())?;
        
        println!("   📊 Claude provided {} words of analysis", claude_response.split_whitespace().count());
        println!("   📑 Identified {} distinct sections", parsed_analysis.len());
        println!("   ✅ Claude response processed and structured");
        
        Ok(context)
    }
}

/// Parse Claude's response into structured sections
fn parse_claude_response(response: &str) -> Vec<serde_json::Value> {
    let mut sections = Vec::new();
    let mut current_section = String::new();
    let mut current_title = String::new();
    
    for line in response.lines() {
        let trimmed = line.trim();
        
        // Detect section headers (lines with **header** or ## header format)
        if (trimmed.starts_with("**") && trimmed.ends_with("**")) || trimmed.starts_with("##") {
            // Save previous section if it exists
            if !current_title.is_empty() && !current_section.is_empty() {
                sections.push(json!({
                    "title": current_title.trim(),
                    "content": current_section.trim(),
                    "word_count": current_section.split_whitespace().count()
                }));
            }
            
            // Start new section
            current_title = trimmed
                .trim_start_matches("**")
                .trim_end_matches("**")
                .trim_start_matches("##")
                .trim()
                .to_string();
            current_section.clear();
        } else {
            // Add to current section
            if !trimmed.is_empty() {
                current_section.push_str(line);
                current_section.push('\n');
            }
        }
    }
    
    // Add the last section
    if !current_title.is_empty() && !current_section.is_empty() {
        sections.push(json!({
            "title": current_title.trim(),
            "content": current_section.trim(),
            "word_count": current_section.split_whitespace().count()
        }));
    }
    
    // If no sections were found, treat the entire response as one section
    if sections.is_empty() {
        sections.push(json!({
            "title": "Analysis",
            "content": response.trim(),
            "word_count": response.split_whitespace().count()
        }));
    }
    
    sections
}

/// Assess the quality of Claude's analysis
fn assess_analysis_quality(response: &str) -> serde_json::Value {
    let word_count = response.split_whitespace().count();
    let sentence_count = response.split('.').count().saturating_sub(1);
    let paragraph_count = response.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    
    // Quality indicators
    let has_specific_examples = response.to_lowercase().contains("example") || 
                               response.to_lowercase().contains("instance") ||
                               response.to_lowercase().contains("for example");
    
    let has_structured_analysis = response.contains("**") || response.contains("##");
    
    let has_evidence_citations = response.contains("evidence") || 
                                response.contains("according to") ||
                                response.contains("the document states");
    
    let depth_score = calculate_depth_score(response);
    let clarity_score = calculate_clarity_score(response, word_count, sentence_count);
    
    json!({
        "depth_score": depth_score,
        "clarity_score": clarity_score,
        "structure_score": if has_structured_analysis { 0.9 } else { 0.5 },
        "evidence_score": if has_evidence_citations { 0.8 } else { 0.4 },
        "example_usage": has_specific_examples,
        "overall_quality": (depth_score + clarity_score) / 2.0,
        "quality_indicators": {
            "specific_examples": has_specific_examples,
            "structured_format": has_structured_analysis,
            "evidence_based": has_evidence_citations,
            "comprehensive_length": word_count > 200
        }
    })
}

fn calculate_depth_score(response: &str) -> f64 {
    let response_lower = response.to_lowercase();
    let depth_indicators = [
        "because", "therefore", "however", "furthermore", "moreover",
        "in contrast", "analysis", "insight", "implication", "significance",
        "complexity", "nuance", "sophisticated", "comprehensive"
    ];
    
    let indicator_count = depth_indicators.iter()
        .map(|&indicator| response_lower.matches(indicator).count())
        .sum::<usize>();
    
    let word_count = response.split_whitespace().count();
    let indicator_density = indicator_count as f64 / word_count.max(1) as f64;
    
    (indicator_density * 100.0).min(1.0)
}

fn calculate_clarity_score(response: &str, word_count: usize, sentence_count: usize) -> f64 {
    if sentence_count == 0 { return 0.0; }
    
    let avg_sentence_length = word_count as f64 / sentence_count as f64;
    let clarity_score = if avg_sentence_length <= 20.0 { 0.9 }
                       else if avg_sentence_length <= 30.0 { 0.7 }
                       else { 0.5 };
    
    // Bonus for good structure
    let structure_bonus = if response.contains("\n\n") { 0.1 } else { 0.0 };
    
    (clarity_score + structure_bonus).min(1.0)
}

fn calculate_reading_metrics(response: &str) -> serde_json::Value {
    let word_count = response.split_whitespace().count();
    let reading_time = (word_count as f64 / 200.0).ceil() as u32; // 200 wpm average
    let sentence_count = response.split('.').count().saturating_sub(1);
    
    json!({
        "word_count": word_count,
        "estimated_reading_time_minutes": reading_time,
        "sentence_count": sentence_count,
        "average_sentence_length": if sentence_count > 0 { word_count as f64 / sentence_count as f64 } else { 0.0 },
        "reading_level": classify_reading_level(word_count, sentence_count)
    })
}

fn classify_reading_level(word_count: usize, sentence_count: usize) -> String {
    if sentence_count == 0 { return "Unknown".to_string(); }
    
    let avg_sentence_length = word_count as f64 / sentence_count as f64;
    
    if avg_sentence_length <= 15.0 {
        "Elementary"
    } else if avg_sentence_length <= 20.0 {
        "Intermediate"
    } else if avg_sentence_length <= 25.0 {
        "Advanced"
    } else {
        "Expert"
    }.to_string()
}

fn classify_analysis_depth(response: &str) -> String {
    let word_count = response.split_whitespace().count();
    let response_lower = response.to_lowercase();
    
    let depth_keywords = response_lower.matches("analysis").count() +
                        response_lower.matches("insight").count() +
                        response_lower.matches("implication").count();
    
    if word_count > 500 && depth_keywords > 3 {
        "Deep"
    } else if word_count > 200 && depth_keywords > 1 {
        "Moderate"
    } else {
        "Surface"
    }.to_string()
}

fn estimate_processing_time(document_data: &serde_json::Value) -> String {
    if let Some(word_count) = document_data["document_metadata"]["word_count"].as_u64() {
        let time_estimate = (word_count as f64 / 100.0).ceil() as u32; // Rough estimate
        format!("{}-{} seconds", time_estimate, time_estimate + 5)
    } else {
        "2-10 seconds".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkflowError> {
    println!("🚀 Anthropic Claude Agent Integration Example");
    println!("=".repeat(60));
    println!("This example demonstrates sophisticated document analysis using Claude models.\n");
    
    // Check for API key
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        println!("❌ Error: ANTHROPIC_API_KEY environment variable not set");
        println!("   Please set your Anthropic API key:");
        println!("   export ANTHROPIC_API_KEY=\"your-anthropic-api-key\"");
        return Ok(());
    }
    
    // Create different Claude model configurations
    println!("📦 Creating Claude agent configurations...");
    
    let opus_config = AgentConfig {
        system_prompt: "You are Claude, an expert analyst with deep knowledge across multiple domains. You excel at nuanced analysis, critical thinking, and providing comprehensive insights. Your responses are thorough, well-structured, and backed by careful reasoning. You consider multiple perspectives and provide balanced, thoughtful analysis.".to_string(),
        model_provider: ModelProvider::Anthropic,
        model_name: "claude-3-opus-20240229".to_string(),
        mcp_server_uri: None,
    };
    
    let sonnet_config = AgentConfig {
        system_prompt: "You are Claude, a knowledgeable analyst who provides clear, well-organized analysis. You balance depth with accessibility, ensuring your insights are both comprehensive and easy to understand. You structure your responses logically and provide practical, actionable insights.".to_string(),
        model_provider: ModelProvider::Anthropic,
        model_name: "claude-3-sonnet-20240229".to_string(),
        mcp_server_uri: None,
    };
    
    // Create workflow nodes
    let preprocessor = DocumentPreprocessorNode;
    let opus_agent = AnthropicAgentNode::new(opus_config);
    let sonnet_agent = AnthropicAgentNode::new(sonnet_config);
    let processor = ClaudeResponseProcessorNode;
    
    println!("   ✅ Created Claude Opus and Sonnet agents");
    println!("   ✅ Created document preprocessing and response processing nodes\n");
    
    // Test documents with different characteristics
    let test_documents = vec![
        DocumentAnalysisRequest {
            title: "The Impact of Artificial Intelligence on Modern Healthcare Systems".to_string(),
            content: "The integration of artificial intelligence (AI) into healthcare systems represents one of the most significant technological advances of the 21st century. This transformation is reshaping how medical professionals diagnose diseases, treat patients, and manage healthcare operations. Machine learning algorithms are now capable of analyzing medical images with accuracy levels that often exceed human specialists. For instance, deep learning models have demonstrated remarkable success in detecting diabetic retinopathy in retinal photographs, identifying skin cancer in dermatological images, and spotting early signs of Alzheimer's disease in brain scans.\n\nBeyond diagnostic applications, AI is revolutionizing drug discovery and development processes. Traditional pharmaceutical research can take 10-15 years and cost billions of dollars to bring a single drug to market. AI-powered platforms are dramatically reducing these timelines by predicting molecular behavior, identifying promising drug compounds, and optimizing clinical trial designs. Companies like DeepMind have made breakthrough discoveries in protein folding prediction, which could accelerate the development of new treatments for previously intractable diseases.\n\nHowever, the implementation of AI in healthcare also presents significant challenges and ethical considerations. Issues of data privacy, algorithmic bias, and the need for regulatory frameworks are paramount. Healthcare AI systems must be transparent, explainable, and fair across diverse patient populations. There are concerns about the potential for AI to exacerbate existing healthcare disparities if not carefully designed and implemented. Additionally, the integration of AI tools requires substantial investment in infrastructure, training, and change management within healthcare organizations.\n\nLooking forward, the future of AI in healthcare appears promising but requires careful navigation of technological, ethical, and regulatory landscapes. Success will depend on collaboration between technologists, healthcare professionals, policymakers, and patient advocacy groups to ensure that AI serves to improve healthcare outcomes for all patients while maintaining the human touch that is so essential to medical care.".to_string(),
            document_type: "research_paper".to_string(),
            analysis_goals: vec!["summarize".to_string(), "extract_insights".to_string(), "identify_themes".to_string()],
            target_audience: "healthcare professionals".to_string(),
        },
        DocumentAnalysisRequest {
            title: "Digital Transformation Strategy for Mid-Market Companies".to_string(),
            content: "In today's rapidly evolving business landscape, digital transformation has moved from being a competitive advantage to a necessity for survival. Mid-market companies, typically defined as organizations with annual revenues between $10 million and $1 billion, face unique challenges in their digital transformation journeys. Unlike large enterprises with substantial resources or small startups built with digital-first approaches, mid-market companies must balance operational constraints with transformation imperatives.\n\nThe foundation of successful digital transformation for mid-market companies lies in strategic prioritization. These organizations cannot afford to pursue every technological opportunity simultaneously. Instead, they must identify high-impact, low-complexity initiatives that can deliver quick wins while building momentum for larger transformations. Customer experience improvements, process automation, and data analytics initiatives often provide the best starting points, offering measurable returns on investment within 6-12 months.\n\nCloud adoption represents perhaps the most critical enabler of digital transformation for mid-market companies. Cloud platforms provide access to enterprise-grade technologies without the capital expenditure and infrastructure complexity traditionally required. This democratization of technology allows mid-market companies to compete with larger rivals while maintaining operational flexibility. However, cloud migration requires careful planning, security considerations, and change management to ensure successful adoption.\n\nThe human element remains the most challenging aspect of digital transformation. Mid-market companies often have deeply embedded processes and cultural resistance to change. Success requires strong leadership commitment, comprehensive training programs, and clear communication about the benefits of transformation initiatives. Companies must invest in upskilling existing employees while also attracting new talent with digital expertise.".to_string(),
            document_type: "business_report".to_string(),
            analysis_goals: vec!["summarize".to_string(), "recommendations".to_string(), "assess_quality".to_string()],
            target_audience: "business executives".to_string(),
        },
        DocumentAnalysisRequest {
            title: "The Quantum Garden".to_string(),
            content: "The old physicist had spent forty years studying the fundamental nature of reality, but nothing had prepared her for what she discovered in her backyard garden that Tuesday morning. The roses were blooming in patterns that defied every law of botany she knew, their petals arranged in impossible geometric configurations that seemed to shift and change when observed directly.\n\nDr. Elena Vasquez knelt beside the peculiar flowers, her weathered hands trembling as she reached for her measurement tools. The electromagnetic readings were off the charts, and the quantum field fluctuations suggested something far beyond normal botanical phenomena. Each rose appeared to exist in multiple states simultaneously, their colors shifting between spectrums visible and invisible to the human eye.\n\n'Schrödinger's roses,' she whispered to herself, remembering the famous thought experiment about the cat that existed in a superposition of states until observed. But these flowers were real, growing from ordinary soil, fed by ordinary water, yet somehow operating according to quantum mechanical principles that should only apply at the subatomic level.\n\nAs days passed, Elena documented the phenomenon with scientific rigor, but also with growing wonder. The garden seemed to respond to observation, to thought, perhaps even to emotion. When she approached with skepticism, the quantum effects diminished. When she came with openness and curiosity, the garden bloomed with impossible beauty, showing her glimpses of realities that existed parallel to her own.\n\nThe implications were staggering. If consciousness could influence quantum states at the macroscopic level, if observation could alter reality in measurable ways, then the boundaries between physics and philosophy, between science and magic, were far more permeable than anyone had imagined.".to_string(),
            document_type: "creative_writing".to_string(),
            analysis_goals: vec!["identify_themes".to_string(), "critique".to_string(), "assess_quality".to_string()],
            target_audience: "literary critics".to_string(),
        },
    ];
    
    // Process each document with both Claude models
    for (i, document) in test_documents.into_iter().enumerate() {
        println!("🔄 Document {} - {}", i + 1, document.title);
        println!("   📄 Type: {}", document.document_type);
        println!("   📊 Length: {} words", document.content.split_whitespace().count());
        
        // Process with Claude Opus
        println!("\n   🧠 Analyzing with Claude Opus...");
        let mut context = TaskContext::new(
            "claude_document_analysis".to_string(),
            serde_json::to_value(&document)?
        );
        
        context = preprocessor.process(context)?;
        context = opus_agent.process(context)?;
        context = processor.process(context)?;
        
        // Display Opus results
        if let Some(result) = context.get_node_data::<serde_json::Value>("claude_analysis")? {
            println!("      📊 Claude Opus Analysis:");
            display_claude_analysis(&result, "      ");
        }
        
        // Process with Claude Sonnet for comparison
        println!("\n   🧠 Analyzing with Claude Sonnet...");
        let mut context2 = TaskContext::new(
            "claude_document_analysis_sonnet".to_string(),
            serde_json::to_value(&document)?
        );
        
        context2 = preprocessor.process(context2)?;
        context2 = sonnet_agent.process(context2)?;
        context2 = processor.process(context2)?;
        
        // Display Sonnet results
        if let Some(result) = context2.get_node_data::<serde_json::Value>("claude_analysis")? {
            println!("      📊 Claude Sonnet Analysis:");
            display_claude_analysis(&result, "      ");
        }
        
        println!("   ✅ Document {} analysis completed\n", i + 1);
    }
    
    println!("🎉 Anthropic Claude Agent Integration Example Complete!");
    println!("=".repeat(60));
    println!("🎓 What you learned:");
    println!("   • Setting up Anthropic Claude agents with different models");
    println!("   • Creating sophisticated prompts for document analysis");
    println!("   • Processing long-form content with Claude's capabilities");
    println!("   • Parsing and structuring Claude's detailed responses");
    println!("   • Comparing Claude Opus vs Sonnet performance");
    println!("   • Building document analysis workflows");
    println!();
    println!("💡 Claude's strengths:");
    println!("   • Excellent for long-form content analysis");
    println!("   • Strong reasoning and critical thinking capabilities");
    println!("   • Nuanced understanding of complex topics");
    println!("   • Well-structured, comprehensive responses");
    println!("   • Larger context windows for extensive documents");
    println!();
    println!("➡️  Next steps:");
    println!("   • Try Claude Haiku for faster, cost-effective analysis");
    println!("   • Experiment with different document types");
    println!("   • Compare Claude with OpenAI models");
    println!("   • Explore the multi-model.rs example");
    
    Ok(())
}

fn display_claude_analysis(result: &serde_json::Value, indent: &str) {
    if let Some(analysis) = result.get("analysis") {
        // Display quality metrics
        if let Some(quality) = analysis.get("quality_metrics") {
            if let (Some(depth), Some(clarity), Some(overall)) = (
                quality.get("depth_score").and_then(|v| v.as_f64()),
                quality.get("clarity_score").and_then(|v| v.as_f64()),
                quality.get("overall_quality").and_then(|v| v.as_f64())
            ) {
                println!("{}Quality: Overall {:.1}% (Depth: {:.1}%, Clarity: {:.1}%)", 
                    indent, overall * 100.0, depth * 100.0, clarity * 100.0);
            }
        }
        
        // Display reading metrics
        if let Some(reading) = analysis.get("reading_metrics") {
            if let (Some(words), Some(time)) = (
                reading.get("word_count").and_then(|v| v.as_u64()),
                reading.get("estimated_reading_time_minutes").and_then(|v| v.as_u64())
            ) {
                println!("{}Response: {} words, ~{} min read", indent, words, time);
            }
        }
        
        // Display sections
        if let Some(sections) = analysis.get("parsed_sections").and_then(|v| v.as_array()) {
            println!("{}Sections analyzed: {}", indent, sections.len());
            for (i, section) in sections.iter().take(3).enumerate() {
                if let Some(title) = section.get("title").and_then(|v| v.as_str()) {
                    println!("{}  {}. {}", indent, i + 1, title);
                }
            }
            if sections.len() > 3 {
                println!("{}  ... and {} more sections", indent, sections.len() - 3);
            }
        }
        
        // Display processing summary
        if let Some(summary) = result.get("processing_summary") {
            if let Some(depth) = summary.get("analysis_depth").and_then(|v| v.as_str()) {
                println!("{}Analysis depth: {}", indent, depth);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document_preprocessor() {
        let request = DocumentAnalysisRequest {
            title: "Test Document".to_string(),
            content: "This is a test document with some content for analysis.".to_string(),
            document_type: "business_report".to_string(),
            analysis_goals: vec!["summarize".to_string()],
            target_audience: "general".to_string(),
        };
        
        let context = TaskContext::new(
            "test".to_string(),
            serde_json::to_value(request).unwrap()
        );
        
        let node = DocumentPreprocessorNode;
        let result = node.process(context).unwrap();
        
        let analysis: serde_json::Value = result
            .get_node_data("document_analysis")
            .unwrap()
            .unwrap();
        
        assert!(analysis.get("prompt").is_some());
        assert!(analysis.get("document_metadata").is_some());
    }
    
    #[test]
    fn test_claude_response_parsing() {
        let response = "**Summary**\nThis is the summary section.\n\n**Key Insights**\nThese are the insights.";
        let sections = parse_claude_response(response);
        
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0]["title"], "Summary");
        assert_eq!(sections[1]["title"], "Key Insights");
    }
    
    #[test]
    fn test_analysis_quality_assessment() {
        let response = "This is a comprehensive analysis with evidence and examples. Therefore, we can conclude...";
        let quality = assess_analysis_quality(response);
        
        assert!(quality.get("depth_score").is_some());
        assert!(quality.get("clarity_score").is_some());
        assert!(quality.get("overall_quality").is_some());
    }
}