
/*!
# AI System Rust

This is a Rust-based AI system that provides a robust architecture for building and managing AI workflows.

## Overview

This system is organized into several key modules:

- [`api`]: HTTP API endpoints and request handlers
- [`core`]: Core system functionality and workflow management
- [`db`]: Database interactions and data persistence
- [`workflows`]: Implementation of specific AI workflows and demos

## Getting Started

To use this system, you typically start with the API layer to define your endpoints,
utilize the core functionality for workflow management, and implement specific workflows
in the workflows module.

## Example

```rust
use backend::{api, core, workflows};

// Your implementation here
```

## Module Structure

- **api**: Contains all HTTP endpoints and request handlers for the web service
- **core**: Houses the fundamental system components and workflow engine
- **db**: Manages database connections and data persistence
- **workflows**: Contains specific workflow implementations and demonstrations

*/

pub mod api;
pub mod bootstrap;
pub mod core;
pub mod db;
pub mod integrations;
pub mod monitoring;
pub mod workflows;
