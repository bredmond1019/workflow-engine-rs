//! Protection Mechanisms Module
//! 
//! Implements rate limiting and circuit breakers for protecting downstream services
//! and maintaining system stability under load.

pub mod rate_limiter;
pub mod circuit_breaker;

pub use rate_limiter::*;
pub use circuit_breaker::*;