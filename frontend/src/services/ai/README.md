# Workflow Intent Detection - Test 4 Implementation

This directory contains the implementation for Test 4: Workflow Intent Detection following TDD principles.

## Overview

The WorkflowIntentAnalyzer service provides natural language understanding capabilities to detect user intentions related to workflow operations. This is the beginning of AI integration in the frontend.

## Test Coverage

### Test 4a: Detects "create workflow" intent
- ✅ Detects explicit create workflow requests
- ✅ Detects implicit automation requests
- ✅ Recognizes various phrasings for workflow creation
- ✅ Distinguishes between creating new vs. viewing existing workflows

### Test 4b: Identifies workflow type from description
- ✅ Customer Support workflows (HelpScout integration)
- ✅ Knowledge Base workflows (Notion integration)
- ✅ Data Processing workflows
- ✅ Communication workflows (Slack integration)
- ✅ AI/ML workflows (GPT-4, Claude integration)
- ✅ Generic workflows for unknown types

### Test 4c: Extracts key parameters from natural language
- ✅ Trigger parameters (schedule, events)
- ✅ Service parameters (source/destination)
- ✅ Condition parameters (filters, rules)
- ✅ Complex multi-parameter extraction
- ✅ Data transformation parameters

### Test 4d: Handles ambiguous requests with clarification
- ✅ Identifies low-confidence intents
- ✅ Generates clarification questions
- ✅ Provides service options when ambiguous
- ✅ Suggests workflow templates
- ✅ Handles completely vague requests

### Test 4e: Recognizes workflow modification requests
- ✅ Detects modification intent
- ✅ Detects update requests (schedule changes)
- ✅ Detects deletion requests
- ✅ Adding steps to workflows
- ✅ Removing steps from workflows
- ✅ Workflow cloning/duplication
- ✅ Handles ambiguous modification requests

## Architecture

### Core Components

1. **WorkflowIntentAnalyzer Service** (`WorkflowIntentAnalyzer.ts`)
   - Main service for analyzing user messages
   - Returns structured intent data
   - Maintains conversation context
   - Currently throws "Not implemented" (Red phase of TDD)

2. **Intent Types**
   - `CREATE_WORKFLOW` - User wants to create a new workflow
   - `MODIFY_WORKFLOW` - User wants to modify an existing workflow
   - `DELETE_WORKFLOW` - User wants to delete a workflow
   - `CLONE_WORKFLOW` - User wants to duplicate a workflow
   - `VIEW_WORKFLOW` - User wants to view workflow details
   - `EXECUTE_WORKFLOW` - User wants to run a workflow
   - `UNKNOWN` - Intent cannot be determined

3. **Workflow Types**
   - `CUSTOMER_SUPPORT` - Support ticket handling
   - `KNOWLEDGE_BASE` - Documentation management
   - `DATA_PROCESSING` - Data transformation/analysis
   - `COMMUNICATION` - Messaging/notifications
   - `AI_ML` - AI/ML processing
   - `INTEGRATION` - System integration
   - `GENERIC` - General purpose

### React Hook

**useWorkflowIntent** (`hooks/useWorkflowIntent.ts`)
- React hook for using the WorkflowIntentAnalyzer
- Handles loading states and errors
- Debounces rapid analysis calls (300ms)
- Maintains intent history
- Provides context management

## Usage Example

```typescript
import { useWorkflowIntent } from './hooks/useWorkflowIntent';

function ChatInterface() {
  const { 
    currentIntent, 
    isAnalyzing, 
    error, 
    analyzeMessage, 
    clearIntent 
  } = useWorkflowIntent();

  const handleMessage = async (message: string) => {
    await analyzeMessage(message);
    
    if (currentIntent?.type === IntentType.CREATE_WORKFLOW) {
      // Handle workflow creation
      if (currentIntent.needsClarification) {
        // Show clarification questions
        return currentIntent.clarificationQuestions;
      }
      // Proceed with workflow creation
    }
  };

  return (
    // UI components
  );
}
```

## Next Steps

1. **Green Phase**: Implement the actual intent analysis logic
2. **Integration**: Connect to AI service (OpenAI/Anthropic)
3. **Training**: Fine-tune for workflow-specific language
4. **UI Integration**: Add to chat interface
5. **Workflow Templates**: Map intents to actual workflow templates

## Testing

Run the tests:
```bash
# Run WorkflowIntentAnalyzer tests (currently failing - Red phase)
npm test -- src/services/ai/WorkflowIntentAnalyzer.test.ts

# Run useWorkflowIntent hook tests (passing with mocks)
npm test -- src/hooks/useWorkflowIntent.test.ts
```

## TDD Status

- ✅ **Red Phase Complete**: All service tests are failing as expected
- ⏳ **Green Phase**: Ready to implement the service
- ⏳ **Refactor Phase**: Will optimize after implementation