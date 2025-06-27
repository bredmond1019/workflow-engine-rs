// Chat-related type definitions

export interface ChatMessage {
  id: string;
  content: string;
  sender: 'user' | 'assistant';
  timestamp: Date;
}

export interface ChatMessageProps {
  message: ChatMessage;
}

export interface ChatInputProps {
  onSendMessage: (message: string) => void;
  disabled?: boolean;
  placeholder?: string;
  maxLength?: number;
}

export interface ChatContainerProps {
  messages: ChatMessage[];
  isLoading?: boolean;
  onSendMessage: (message: string) => void;
}