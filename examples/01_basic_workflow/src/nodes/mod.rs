//! Custom node implementations for the basic workflow example
//!
//! This module contains three example nodes that demonstrate different
//! aspects of workflow processing:
//!
//! - [`TextInputNode`] - Input validation and preprocessing
//! - [`TextProcessorNode`] - Core processing logic with configurable operations
//! - [`TextOutputNode`] - Result formatting and metadata generation

pub mod text_input;
pub mod text_processor;
pub mod text_output;

pub use text_input::TextInputNode;
pub use text_processor::TextProcessorNode;
pub use text_output::TextOutputNode;