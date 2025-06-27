export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  metadata?: {
    intentType?: string;
    extractedEntities?: Record<string, any>;
    suggestedActions?: string[];
  };
}

export interface ChatConversation {
  id: string;
  title: string;
  messages: ChatMessage[];
  createdAt: Date;
  updatedAt: Date;
  status: 'active' | 'archived';
}

export interface ChatContext {
  currentIntent?: string;
  extractedData?: Record<string, any>;
  workflowSuggestions?: string[];
  confidenceScore?: number;
}