// WorkflowIntentAnalyzer - AI service for analyzing user messages to detect workflow-related intents

export enum IntentType {
  CREATE_WORKFLOW = 'CREATE_WORKFLOW',
  MODIFY_WORKFLOW = 'MODIFY_WORKFLOW',
  DELETE_WORKFLOW = 'DELETE_WORKFLOW',
  CLONE_WORKFLOW = 'CLONE_WORKFLOW',
  VIEW_WORKFLOW = 'VIEW_WORKFLOW',
  EXECUTE_WORKFLOW = 'EXECUTE_WORKFLOW',
  UNKNOWN = 'UNKNOWN'
}

export enum WorkflowType {
  CUSTOMER_SUPPORT = 'CUSTOMER_SUPPORT',
  KNOWLEDGE_BASE = 'KNOWLEDGE_BASE',
  DATA_PROCESSING = 'DATA_PROCESSING',
  COMMUNICATION = 'COMMUNICATION',
  AI_ML = 'AI_ML',
  INTEGRATION = 'INTEGRATION',
  GENERIC = 'GENERIC'
}

export interface WorkflowParameter {
  name: string;
  value: string | number | boolean;
  metadata?: Record<string, any>;
}

export interface WorkflowModification {
  action: 'add' | 'remove' | 'update' | 'replace';
  component: string;
  details: any;
}

export interface WorkflowIntent {
  type: IntentType;
  confidence: number;
  workflowType?: WorkflowType;
  suggestedTemplate?: string;
  parameters: WorkflowParameter[];
  needsClarification: boolean;
  clarificationQuestions?: string[];
  suggestedOptions?: string[];
  suggestions?: Array<{
    template: string;
    description: string;
  }>;
  targetWorkflow?: string;
  sourceWorkflow?: string;
  modifications?: WorkflowModification | WorkflowModification[];
  detectedServices?: string[];
  contextualReference?: boolean;
}

export class WorkflowIntentAnalyzer {
  private context: Map<string, any> = new Map();

  async analyzeIntent(message: string): Promise<WorkflowIntent> {
    // This method will be implemented to analyze the user's message
    // and return the detected intent with all relevant information
    throw new Error('Not implemented');
  }

  clearContext(): void {
    this.context.clear();
  }

  getContext(): Map<string, any> {
    return new Map(this.context);
  }
}