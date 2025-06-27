// Chat feature constants

// Character count thresholds
export const CHARACTER_COUNT = {
  SHOW_THRESHOLD: 0.7, // Show count when 70% of max length reached
  WARNING_THRESHOLD: 0.9, // Show warning when 90% of max length reached
} as const;

// Avatar display values
export const AVATAR = {
  USER: 'U',
  ASSISTANT: 'AI',
} as const;

// Default UI text
export const UI_TEXT = {
  DEFAULT_PLACEHOLDER: 'Type a message...',
  LOADING_MESSAGE: 'AI is thinking...',
  EMPTY_STATE: {
    TITLE: 'Start a conversation',
    SUBTITLE: "I'm here to help you create workflows",
    HELPER: 'Type a message below to get started',
  },
} as const;

// Time formatting options
export const TIME_FORMAT_OPTIONS: Intl.DateTimeFormatOptions = {
  hour: 'numeric',
  minute: '2-digit',
  hour12: true,
} as const;

// Accessibility labels
export const ARIA_LABELS = {
  CHAT_MESSAGES: 'Chat messages',
  CHAT_CONVERSATION: 'Chat conversation',
} as const;

// CSS class prefixes
export const CSS_CLASSES = {
  CHAT_MESSAGE: 'chat-message',
  AVATAR: 'avatar',
  CHARACTER_COUNT: 'character-count',
} as const;

// Markdown language fallback
export const DEFAULT_CODE_LANGUAGE = 'plaintext';

// UI Icons
export const ICONS = {
  SEND: '➤',
  LOADING: '⋯',
} as const;