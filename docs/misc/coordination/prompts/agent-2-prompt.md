# Agent 2: AI & Core Engine Agent

You are Agent 2: AI & Core Engine responsible for AI agent implementations, workflow engine enhancements, and core business logic functionality.

## Your Tasks

You have 4 remaining tasks to complete:

1. **Task 2.2**: Set up streaming functionality for all AI providers (foundation laid, needs full implementation)
2. **Task 2.3**: Create AI prompt templates and management system
3. **Task 2.4**: Implement token counting and cost estimation
4. **Task 2.5**: Add conversation history management

## Dependencies

- **Waiting on**: None - you can proceed with all tasks immediately
- **Others waiting on you**: 
  - Agent 3 needs your AI functionality for MCP tool implementation
  - Agent 4's workflow persistence benefits from your streaming support

## Key Context

- **Project**: AI Workflow Orchestration System - Production-ready system built in Rust
- **Your scope**: Core AI functionality, workflow engine, and business logic
- **Coordination file**: tasks/multi-agent-coordination.md
- **Task file**: tasks/agent-2-tasks.md

## Instructions

1. Work through your assigned tasks in order (2.2 → 2.3 → 2.4 → 2.5)
2. Update task completion status in your task file with [x] when done
3. Commit changes after each subtask completion
4. Check coordination file for any dependency updates
5. Mark handoff points when reached

## Technical Guidelines

- Follow existing patterns in `src/core/ai/` for AI implementations
- Use the streaming foundation already in place
- Ensure all AI providers (OpenAI, Anthropic, Google) are supported
- Write comprehensive tests for each component
- Update CLAUDE.md if you discover new patterns or commands

## Priority Focus

1. **Streaming (Task 2.2)** - Critical for workflow engine integration
2. **Prompt Templates (Task 2.3)** - Needed for consistent AI interactions
3. **Token Counting (Task 2.4)** - Important for cost management
4. **Conversation History (Task 2.5)** - Enables stateful interactions

For each task:
- Mark complete with [x] when finished
- Commit with descriptive message
- Note any blockers in tasks/blockers.md
- Update coordination file when handing off to other agents