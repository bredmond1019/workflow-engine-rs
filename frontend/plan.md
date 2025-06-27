# Chat-Based Workflow UI - TDD Plan

## Overview
Building a modern, conversational interface for creating AI workflows using Test-Driven Development (TDD) methodology. Each test follows the Red-Green-Refactor cycle.

## Test Specifications

### Phase 1: Core Chat Interface Foundation

#### [x] Test 1: ChatMessage Component
**Purpose**: Display chat messages with proper styling for user vs assistant
```typescript
// Test cases:
- [x] 1a. Renders user message with correct styling
- [x] 1b. Renders assistant message with correct styling  
- [x] 1c. Displays timestamp
- [x] 1d. Shows avatar/icon for message sender
- [x] 1e. Handles markdown content rendering
```

#### [x] Test 2: ChatInput Component
**Purpose**: Handle user input and message submission
```typescript
// Test cases:
- [x] 2a. Renders input field with placeholder
- [x] 2b. Handles text input changes
- [x] 2c. Submits on Enter key press
- [x] 2d. Clears input after submission
- [x] 2e. Disables input while processing
- [x] 2f. Shows character count limit
```

#### [ ] Test 3: ChatContainer Layout
**Purpose**: Container for chat messages with proper scrolling
```typescript
// Test cases:
- [ ] 3a. Renders list of messages in order
- [ ] 3b. Auto-scrolls to bottom on new message
- [ ] 3c. Maintains scroll position when loading history
- [ ] 3d. Shows loading indicator when fetching
- [ ] 3e. Displays empty state when no messages
```

### Phase 2: AI Assistant Integration

#### [ ] Test 4: Workflow Intent Detection
**Purpose**: Parse user input to identify workflow intent
```typescript
// Test cases:
- [ ] 4a. Detects "create workflow" intent
- [ ] 4b. Identifies workflow type from description
- [ ] 4c. Extracts key parameters from natural language
- [ ] 4d. Handles ambiguous requests with clarification
- [ ] 4e. Recognizes workflow modification requests
```

#### [ ] Test 5: Dynamic Form Generation
**Purpose**: Generate form fields based on conversation
```typescript
// Test cases:
- [ ] 5a. Creates text input from chat context
- [ ] 5b. Generates select dropdown from options
- [ ] 5c. Builds multi-step form progressively
- [ ] 5d. Validates fields through conversation
- [ ] 5e. Shows form preview alongside chat
```

#### [ ] Test 6: Workflow Preview
**Purpose**: Visual representation of workflow being built
```typescript
// Test cases:
- [ ] 6a. Renders workflow nodes from chat data
- [ ] 6b. Updates preview in real-time
- [ ] 6c. Shows connections between nodes
- [ ] 6d. Highlights current configuration step
- [ ] 6e. Allows basic interaction with preview
```

### Phase 3: Advanced Features

#### [ ] Test 7: Context-Aware Suggestions
**Purpose**: Provide smart suggestions based on context
```typescript
// Test cases:
- [ ] 7a. Shows relevant quick reply chips
- [ ] 7b. Suggests next logical steps
- [ ] 7c. Recommends templates based on intent
- [ ] 7d. Updates suggestions as context changes
- [ ] 7e. Handles suggestion selection
```

#### [ ] Test 8: Multi-Step Workflow Builder
**Purpose**: Guide through complex workflow creation
```typescript
// Test cases:
- [ ] 8a. Tracks workflow creation progress
- [ ] 8b. Allows navigation between steps
- [ ] 8c. Saves partial progress
- [ ] 8d. Shows step indicators
- [ ] 8e. Handles step validation
```

#### [ ] Test 9: Voice Input Integration
**Purpose**: Enable voice commands for accessibility
```typescript
// Test cases:
- [ ] 9a. Toggles voice recording on/off
- [ ] 9b. Shows recording indicator
- [ ] 9c. Converts speech to text
- [ ] 9d. Handles voice command errors
- [ ] 9e. Provides audio feedback
```

## Implementation Guidelines

### TDD Cycle for Each Test
1. **Red**: Write failing test first
2. **Green**: Write minimal code to pass
3. **Refactor**: Improve code structure (Tidy First)

### Commit Convention
- Test commits: `test: Add test for [component] - [test case]`
- Implementation commits: `feat: Implement [component] to pass [test]`
- Refactor commits: `refactor: [Tidy First] Improve [component] structure`

### File Naming
```
src/features/chat/components/[Component]/
├── [Component].tsx
├── [Component].test.tsx
├── [Component].module.css
└── index.ts
```

### Testing Tools
- Jest + React Testing Library
- MSW for API mocking
- Playwright for E2E tests

## Progress Tracking
Mark tests with [x] when complete. Each test should have:
- [ ] Test written (Red)
- [ ] Implementation (Green)
- [ ] Refactoring (Tidy First)
- [ ] All sub-tests passing