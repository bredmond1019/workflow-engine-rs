//! # Demo Timing Constants
//!
//! This module provides standardized timing constants for creating consistent and smooth
//! user experiences across all workflow demonstrations. The timing constants are carefully
//! calibrated to provide optimal pacing for visual feedback, allowing users to follow
//! the execution flow while maintaining engagement.
//!
//! ## Timing Categories
//!
//! The timing constants are organized into several categories based on the type of operation
//! and the desired user experience:
//!
//! ### Quick Operations (< 500ms)
//! For rapid transitions and immediate feedback where delays would feel unnatural.
//!
//! ### Medium Operations (500ms - 1s)
//! For processing steps that benefit from visual confirmation but shouldn't slow down the demo.
//!
//! ### Long Operations (1s+)
//! For major transitions, complex operations, or when users need time to read detailed output.
//!
//! ## Usage Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use ai_architecture_workflows::demos::timing::*;
//! use tokio::time::sleep;
//!
//! pub async fn example_demo() {
//!     println!("Starting demo...");
//!     sleep(WORKFLOW_START_PAUSE).await;
//!     
//!     println!("Processing node...");
//!     sleep(NODE_PROCESSING_PAUSE).await;
//!     
//!     println!("Demo completed!");
//!     sleep(DEMO_TRANSITION_PAUSE).await;
//! }
//! ```
//!
//! ### Combining Timing Constants
//!
//! ```rust
//! use ai_architecture_workflows::demos::timing::*;
//! use tokio::time::sleep;
//! use std::time::Duration;
//!
//! pub async fn custom_timing_demo() {
//!     // Custom timing for special operations
//!     let extended_pause = MEDIUM_PAUSE + Duration::from_millis(200);
//!     sleep(extended_pause).await;
//!     
//!     // Conditional timing based on operation complexity
//!     let timing = if complex_operation {
//!         LONG_PAUSE
//!     } else {
//!         QUICK_PAUSE
//!     };
//!     sleep(timing).await;
//! }
//! ```
//!
//! ### Database Operation Timing
//!
//! ```rust
//! use ai_architecture_workflows::demos::timing::*;
//! use tokio::time::sleep;
//!
//! pub async fn database_demo() {
//!     println!("Creating database event...");
//!     sleep(DATABASE_OPERATION_PAUSE).await;
//!     
//!     println!("Storing event...");
//!     sleep(DATABASE_OPERATION_PAUSE).await;
//!     
//!     println!("Database operations completed!");
//! }
//! ```
//!
//! ### Knowledge Search Timing
//!
//! ```rust
//! use ai_architecture_workflows::demos::timing::*;
//! use tokio::time::sleep;
//!
//! pub async fn knowledge_search_demo() {
//!     println!("Searching Notion database...");
//!     sleep(KNOWLEDGE_SEARCH_PAUSE).await;
//!     
//!     println!("Searching HelpScout articles...");
//!     sleep(KNOWLEDGE_SEARCH_PAUSE).await;
//!     
//!     println!("Search completed!");
//! }
//! ```
//!
//! ### MCP Tool Call Timing
//!
//! ```rust
//! use ai_architecture_workflows::demos::timing::*;
//! use tokio::time::sleep;
//!
//! pub async fn mcp_tool_demo() {
//!     println!("Preparing MCP tool call...");
//!     sleep(CONFIGURATION_PAUSE).await;
//!     
//!     println!("Executing MCP tool...");
//!     sleep(MCP_TOOL_CALL_PAUSE).await;
//!     
//!     println!("Processing tool results...");
//!     sleep(OPERATION_PAUSE).await;
//! }
//! ```
//!
//! ## Timing Guidelines
//!
//! ### Visual Feedback Principles
//!
//! - **Immediate Operations**: Use `QUICK_PAUSE` for operations that complete instantly
//! - **Processing Feedback**: Use `NODE_PROCESSING_PAUSE` for individual node execution
//! - **User Reading Time**: Use `SECTION_PAUSE` when displaying important information
//! - **Major Transitions**: Use `SCENARIO_PAUSE` between different demo scenarios
//!
//! ### Customization Guidelines
//!
//! ```rust
//! // Good: Extending existing constants for special cases
//! let extended_operation = OPERATION_PAUSE + Duration::from_millis(200);
//!
//! // Good: Conditional timing based on operation complexity
//! let timing = match operation_type {
//!     OperationType::Simple => SHORT_PAUSE,
//!     OperationType::Complex => MEDIUM_PAUSE,
//!     OperationType::Heavy => LONG_PAUSE,
//! };
//!
//! // Avoid: Hardcoded timing values
//! // sleep(Duration::from_millis(500)).await; // Don't do this
//! ```
//!
//! ### Performance Considerations
//!
//! The timing constants are designed to balance user experience with demo efficiency:
//!
//! - **Total Demo Time**: Complete demo suite runs in approximately 8-12 minutes
//! - **Individual Scenarios**: Each scenario completes in 30-60 seconds
//! - **Node Processing**: Individual nodes show timing of 150-800ms
//! - **Transitions**: Major transitions take 1-2 seconds for user orientation
//!
//! ## Accessibility Considerations
//!
//! The timing constants consider users with different needs:
//!
//! - **Reading Speed**: Longer pauses when significant text output is displayed
//! - **Processing Time**: Adequate time to understand what's happening
//! - **Visual Tracking**: Consistent timing helps users follow the execution flow
//! - **Cognitive Load**: Pauses prevent information overload
//!
//! ## Testing and Validation
//!
//! When modifying timing constants, consider:
//!
//! ### User Testing Feedback
//! ```rust
//! // Test with different audiences
//! let timing = match audience_level {
//!     AudienceLevel::Beginner => LONG_PAUSE,      // More time to read
//!     AudienceLevel::Intermediate => MEDIUM_PAUSE, // Standard timing
//!     AudienceLevel::Expert => SHORT_PAUSE,       // Faster progression
//! };
//! ```
//!
//! ### Performance Impact
//! ```rust
//! // Measure total demo execution time
//! let start = std::time::Instant::now();
//! run_all_demos().await;
//! let total_time = start.elapsed();
//! println!("Total demo time: {:?}", total_time);
//! ```
//!
//! ## Environment-Based Timing
//!
//! For different deployment environments:
//!
//! ```rust
//! use std::env;
//! use std::time::Duration;
//!
//! pub fn get_demo_timing() -> Duration {
//!     match env::var("DEMO_SPEED").as_deref() {
//!         Ok("fast") => QUICK_PAUSE,
//!         Ok("slow") => LONG_PAUSE,
//!         _ => MEDIUM_PAUSE, // Default
//!     }
//! }
//! ```
//!
//! ## Debugging Timing Issues
//!
//! When demos feel too fast or too slow:
//!
//! ```bash
//! # Run with timing debug info
//! RUST_LOG=ai_architecture_workflows::demos::timing=debug cargo run
//!
//! # Test specific timing values
//! DEMO_SPEED=fast cargo run    # Uses QUICK_PAUSE variants
//! DEMO_SPEED=slow cargo run    # Uses LONG_PAUSE variants
//! ```
//!
//! ## Related Constants
//!
//! See also:
//! - [`std::time::Duration`] for creating custom timing values
//! - [`tokio::time::sleep`] for implementing pauses in async code
//! - Demo modules for usage examples in context

use std::time::Duration;

// Short pauses for quick operations
pub const QUICK_PAUSE: Duration = Duration::from_millis(200);
pub const SHORT_PAUSE: Duration = Duration::from_millis(400);
pub const NODE_PROCESSING_PAUSE: Duration = Duration::from_millis(300);

// Medium pauses for processing steps
pub const MEDIUM_PAUSE: Duration = Duration::from_millis(500);
pub const OPERATION_PAUSE: Duration = Duration::from_millis(600);
pub const SECTION_PAUSE: Duration = Duration::from_millis(800);

// Long pauses for major transitions
pub const LONG_PAUSE: Duration = Duration::from_secs(1);
pub const SCENARIO_PAUSE: Duration = Duration::from_secs(2);
pub const DEMO_TRANSITION_PAUSE: Duration = Duration::from_secs(1);

// Specialized timing for different demo types
pub const WORKFLOW_START_PAUSE: Duration = Duration::from_secs(1);
pub const MCP_TOOL_CALL_PAUSE: Duration = Duration::from_millis(500);
pub const DATABASE_OPERATION_PAUSE: Duration = Duration::from_millis(500);
pub const KNOWLEDGE_SEARCH_PAUSE: Duration = Duration::from_millis(700);
pub const CONFIGURATION_PAUSE: Duration = Duration::from_millis(400);

// New timing constants for enhanced readability
pub const DEMO_PAUSE: Duration = Duration::from_millis(600);
pub const READING_PAUSE: Duration = Duration::from_secs(1);
pub const NODE_START_PAUSE: Duration = Duration::from_millis(300);
pub const NODE_WORK_PAUSE: Duration = Duration::from_millis(500);
pub const NODE_COMPLETE_PAUSE: Duration = Duration::from_millis(300);
pub const NODE_RESULT_PAUSE: Duration = Duration::from_millis(700);