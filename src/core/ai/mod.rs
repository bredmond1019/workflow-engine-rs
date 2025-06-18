//! # AI Module
//!
//! This module provides AI-related functionality including prompt templates,
//! agent management, token counting, cost estimation, and AI service integrations.

pub mod templates;
pub mod tokens;

// Re-export commonly used types
pub use templates::{
    Template, TemplateEngine, TemplateManager, TemplateError,
    TemplateVariables, OutputFormat, render_template,
};

pub use tokens::{
    Model, Provider, TokenUsage, CostBreakdown, UsageRecord,
    TokenCounter, TokenCounterBuilder, PricingEngine, PricingConfig,
    UsageAnalytics, AnalyticsConfig, BudgetLimits, LimitConfig,
    TokenError, TokenResult,
};