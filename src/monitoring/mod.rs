/*!
# Monitoring Module

This module provides comprehensive monitoring and observability infrastructure
for the AI Workflow System, including Prometheus metrics, distributed tracing,
correlation ID management, and structured logging.

Task 3.0: Implement Monitoring and Debugging Infrastructure
*/

pub mod metrics;
pub mod correlation;
pub mod tracing;
pub mod logging;