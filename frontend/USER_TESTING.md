# User Testing Guide - Chat-Based Workflow UI

This guide will help you validate all the components we've built for the chat-based workflow UI. Follow each section to test individual components and their integrations.

## Prerequisites

### 1. Backend Services Setup

Before testing the frontend, ensure backend services are running:

```bash
# Start PostgreSQL (if not using Docker)
pg_ctl start  # or systemctl start postgresql

# Start all backend services with Docker
docker-compose up -d

# Or start services individually:
# Terminal 1: Main API
cargo run --bin workflow-engine

# Terminal 2: GraphQL Gateway
cargo run --bin graphql-gateway

# Terminal 3: MCP Test Servers
./scripts/start_test_servers.sh
```

#### Backend Health Checks
Verify services are running:
- [ ] Main API: `curl http://localhost:8080/health` returns `{"status":"healthy"}`
- [ ] GraphQL Gateway: Open http://localhost:4000/graphql (should see playground)
- [ ] MCP Servers: `curl http://localhost:8001/health` returns `{"status":"ok"}`
- [ ] Database: `psql -h localhost -U aiworkflow -d ai_workflow_db -c "SELECT 1"`

### 2. Start the Frontend Development Server
```bash
cd frontend
npm install  # If not already done
npm run dev
```

The application should start at `http://localhost:5173`

### 3. Configure Frontend Environment

Create `frontend/.env` file:
```env
VITE_API_URL=http://localhost:8080
VITE_GRAPHQL_URL=http://localhost:4000/graphql
VITE_WS_URL=ws://localhost:8084/ws
```

### 4. Create a Test Page
Since we've built individual components but haven't integrated them into a full page yet, create a temporary test page:

Create `src/features/chat/ChatWorkflowPage.tsx`:
```tsx
import React, { useState } from 'react';
import { ChatContainer } from './components/ChatContainer';
import { DynamicForm } from './components/DynamicForm';
import { WorkflowPreview } from './components/WorkflowPreview';
import { useWorkflowIntent } from '../../hooks/useWorkflowIntent';
import type { ChatMessage } from './types/chat';
import './ChatWorkflowPage.css';

export const ChatWorkflowPage: React.FC = () => {
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      id: '1',
      sender: 'assistant',
      content: 'Hello! I can help you create AI workflows. What would you like to automate today?',
      timestamp: new Date()
    }
  ]);
  const [isLoading, setIsLoading] = useState(false);
  const { currentIntent, analyzeMessage } = useWorkflowIntent();

  const handleSendMessage = async (content: string) => {
    // Add user message
    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      sender: 'user',
      content,
      timestamp: new Date()
    };
    setMessages(prev => [...prev, userMessage]);

    // Analyze intent
    setIsLoading(true);
    await analyzeMessage(content);
    
    // Simulate assistant response
    setTimeout(() => {
      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        sender: 'assistant',
        content: 'I understand you want to create a workflow. Let me help you set that up.',
        timestamp: new Date()
      };
      setMessages(prev => [...prev, assistantMessage]);
      setIsLoading(false);
    }, 1000);
  };

  return (
    <div className="chat-workflow-page">
      <div className="chat-section">
        <ChatContainer
          messages={messages}
          isLoading={isLoading}
          onSendMessage={handleSendMessage}
        />
      </div>
      <div className="preview-section">
        <WorkflowPreview
          intent={currentIntent}
          currentStep={1}
        />
        {currentIntent && (
          <DynamicForm
            messages={messages}
            intent={currentIntent}
            onSubmit={(values) => console.log('Form submitted:', values)}
          />
        )}
      </div>
    </div>
  );
};
```

Create `src/features/chat/ChatWorkflowPage.css`:
```css
.chat-workflow-page {
  display: flex;
  height: 100vh;
  background: #0a0a0a;
  color: #fff;
}

.chat-section {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.preview-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  overflow-y: auto;
}
```

Add route in your main App component or router configuration.

## Component Testing

### A. Core Chat Interface

#### 1. ChatMessage Component
- [ ] **User Messages**: Check that user messages appear on the right with blue gradient background
- [ ] **Assistant Messages**: Check that assistant messages appear on the left with purple gradient background
- [ ] **Timestamps**: Verify each message shows a timestamp (e.g., "2:30 PM")
- [ ] **Avatars**: User messages should show "You" label, assistant messages show "AI" label
- [ ] **Markdown Rendering**: Test with markdown content:
  ```
  Test message with **bold**, *italic*, and `code`
  - List item 1
  - List item 2
  ```

#### 2. ChatInput Component
- [ ] **Text Input**: Can type in the textarea
- [ ] **Enter Key**: Pressing Enter sends the message (Shift+Enter for new line)
- [ ] **Send Button**: Clicking send button submits the message
- [ ] **Clear After Send**: Input clears after sending
- [ ] **Character Count**: Shows "X/2000" character count
- [ ] **Character Limit**: Cannot type beyond 2000 characters
- [ ] **Disabled State**: Input is disabled while loading

#### 3. ChatContainer Component
- [ ] **Message Display**: All messages appear in chronological order
- [ ] **Auto-scroll**: New messages trigger auto-scroll to bottom
- [ ] **Loading Indicator**: Shows "AI is thinking..." when loading
- [ ] **Empty State**: Shows "Start a conversation..." when no messages
- [ ] **Scroll Behavior**: Can manually scroll up to see history

### B. AI Workflow Intent Analysis

#### 1. Intent Detection Tests
Type these messages and verify the intent is detected:

- [ ] **Create Workflow**: "I want to create a new workflow" 
  - Should detect CREATE_WORKFLOW intent
- [ ] **Specific Workflow**: "Create a customer support automation using HelpScout"
  - Should detect workflow type: CUSTOMER_SUPPORT
- [ ] **With Parameters**: "Build a workflow that runs every Monday at 9 AM"
  - Should extract schedule parameters
- [ ] **Ambiguous Request**: "I want to create something"
  - Should show needsClarification = true

#### 2. Service Detection
- [ ] **HelpScout**: Mention "helpscout" or "help scout" - should be detected
- [ ] **Slack**: Mention "slack" or "send to slack" - should be detected
- [ ] **Notion**: Mention "notion" or "knowledge base" - should be detected
- [ ] **AI Services**: Mention "GPT-4" or "Claude" - should detect AI_ML type

### C. Dynamic Form Generation

#### 1. Form Field Generation
Based on conversation, verify these fields appear:

- [ ] **Text Fields**: Basic text inputs for workflow name, description
- [ ] **Email Fields**: Email input with @ validation
- [ ] **Select Dropdowns**: Service selection dropdowns
- [ ] **Date/Time Fields**: Schedule configuration
- [ ] **Checkboxes**: Boolean options
- [ ] **Radio Buttons**: Single choice options

#### 2. Multi-Step Forms
- [ ] **Step Indicator**: Shows "Step 1 of 3" etc.
- [ ] **Navigation**: Can click Next/Previous buttons
- [ ] **Step Validation**: Cannot proceed without filling required fields
- [ ] **Submit Button**: Appears on the last step only

#### 3. Form Validation
- [ ] **Required Fields**: Shows "This field is required" error
- [ ] **Email Validation**: Invalid emails show error
- [ ] **Error Clearing**: Errors disappear when corrected
- [ ] **Submit Prevention**: Cannot submit with errors

#### 4. Form Preview
- [ ] **Preview Panel**: Shows on the right side of form
- [ ] **Real-time Updates**: Preview updates as you type
- [ ] **Completion Status**: Shows "50% complete" etc.
- [ ] **Toggle Button**: Can hide/show preview

### D. Workflow Preview Visualization

#### 1. Node Rendering
Verify these node types appear based on conversation:

- [ ] **Trigger Node** (green): For schedules/events
- [ ] **Source Node** (blue): For data sources (HelpScout, etc.)
- [ ] **Condition Node** (yellow): For filters/conditions
- [ ] **Action Node** (purple): For outputs (Slack, etc.)
- [ ] **AI Node** (gradient): For AI processing

#### 2. Visual Features
- [ ] **Connections**: Lines connect nodes showing data flow
- [ ] **Animations**: Active connections show flowing animation
- [ ] **Current Step**: Active node has pulsing highlight
- [ ] **Node States**: Completed (solid), Active (pulsing), Pending (faded)

#### 3. Interactions
- [ ] **Click Nodes**: Clicking a node triggers onNodeClick
- [ ] **Hover Tooltips**: Hovering shows node details
- [ ] **Keyboard Navigation**: Arrow keys move between nodes
- [ ] **Right-click Menu**: Right-click shows context menu
- [ ] **Zoom/Pan**: Can zoom with scroll, pan with drag

## Backend Integration Testing

### API Authentication
Before testing workflows, verify authentication works:

1. **Get JWT Token**:
   ```javascript
   // In browser console
   fetch('http://localhost:8080/auth/token', {
     method: 'POST',
     headers: { 'Content-Type': 'application/json' },
     body: JSON.stringify({ username: 'admin', password: 'admin123' })
   }).then(r => r.json()).then(console.log)
   ```
   - [ ] Returns token object with `token` and `expires_in`

2. **Test Authenticated Request**:
   ```javascript
   // Use token from above
   const token = 'YOUR_TOKEN_HERE';
   fetch('http://localhost:8080/api/v1/workflows', {
     headers: { 'Authorization': `Bearer ${token}` }
   }).then(r => r.json()).then(console.log)
   ```
   - [ ] Returns workflow list (may be empty)

### GraphQL Integration
Test GraphQL queries in the playground (http://localhost:4000/graphql):

1. **Query Workflows**:
   ```graphql
   query {
     workflows {
       id
       name
       description
       status
     }
   }
   ```
   - [ ] Returns workflow data

2. **Create Workflow via GraphQL**:
   ```graphql
   mutation {
     createWorkflow(input: {
       name: "Test Chat Workflow"
       description: "Created from chat UI"
       nodes: []
     }) {
       id
       name
     }
   }
   ```
   - [ ] Creates workflow successfully

### MCP Server Integration
Verify MCP servers respond to chat intents:

- [ ] HelpScout keywords trigger service detection
- [ ] Notion keywords trigger service detection  
- [ ] Slack keywords trigger service detection

## Integration Testing Scenarios

### Scenario 1: Create Customer Support Workflow
1. Type: "I want to create a customer support workflow with HelpScout"
2. Verify:
   - [ ] Intent detected as CREATE_WORKFLOW with CUSTOMER_SUPPORT type
   - [ ] Form shows fields for HelpScout configuration
   - [ ] Preview shows HelpScout source node
3. Continue: "It should check for urgent tickets"
4. Verify:
   - [ ] Condition node appears in preview
   - [ ] Form adds condition/filter fields
5. Continue: "And notify the team on Slack"
6. Verify:
   - [ ] Slack action node appears
   - [ ] Form includes Slack channel field

### Scenario 2: Ambiguous Request Flow
1. Type: "Help me automate something"
2. Verify:
   - [ ] System asks for clarification
   - [ ] Shows workflow type suggestions
3. Select a suggestion and verify flow continues

### Scenario 3: Multi-Step Workflow
1. Type: "Create a data processing workflow that converts CSV to PDF and emails the result"
2. Verify:
   - [ ] Multiple nodes appear (source → transform → action)
   - [ ] Form has multiple steps
   - [ ] Can navigate between form steps

## Performance & Edge Cases

### Performance Tests
- [ ] **Large Conversations**: Add 50+ messages - scrolling remains smooth
- [ ] **Rapid Typing**: Type quickly - intent analysis is debounced
- [ ] **Form Updates**: Rapid form changes don't cause lag

### Edge Cases
- [ ] **Empty Input**: Sending empty message is prevented
- [ ] **Very Long Input**: Character limit (2000) is enforced
- [ ] **No Intent**: Random text shows UNKNOWN intent
- [ ] **Multiple Intents**: "Create and delete workflow" - handles appropriately

## Validation Checklist

Mark each item as it's tested:

### ✅ Working Features
- [ ] Chat message display
- [ ] Message input and sending
- [ ] Auto-scroll behavior
- [ ] Intent detection for basic commands
- [ ] Form field generation
- [ ] Workflow node visualization
- [ ] Basic interactions (click, hover)

### ⚠️ Partially Working
- [ ] Feature: ________________
  - Issue: ________________

### ❌ Not Working
- [ ] Feature: ________________
  - Error: ________________

## Console Checks

Open browser DevTools (F12) and check:
1. **Console Tab**: Any red errors?
2. **Network Tab**: Failed requests?
3. **React DevTools**: Component state updates correctly?

## Common Issues & Solutions

### Issue: Components not rendering
- Check if all imports are correct
- Verify CSS modules are loading
- Ensure test page is properly routed

### Issue: Intent not detecting
- Check WorkflowIntentAnalyzer implementation
- Verify keyword matching logic
- Check for typos in test phrases

### Issue: Form not updating
- Verify intent prop is passed correctly
- Check form field generation logic
- Ensure state updates are working

### Issue: Preview not showing nodes
- Verify intent has required parameters
- Check node creation logic
- Ensure SVG rendering is working

## Notes Section

Use this space to document any specific issues or observations:

```
Date: ___________
Tester: _________

Observations:
- 
- 
- 

Issues Found:
1. 
2. 
3. 

Suggestions:
- 
- 
```

## Next Steps

After testing, report back with:
1. Which sections are fully working
2. Any errors or issues encountered
3. Screenshots of problematic areas (if possible)
4. Console error messages
5. Suggestions for improvements

This will help us prioritize fixes and improvements!