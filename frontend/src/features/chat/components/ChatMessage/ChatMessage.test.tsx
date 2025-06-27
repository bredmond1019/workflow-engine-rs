import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import ChatMessage from './ChatMessage';

describe('ChatMessage Component', () => {
  // Test 1a: Renders user message with correct styling
  test('renders user message with correct styling', () => {
    const userMessage = {
      id: '1',
      content: 'I need to create a workflow',
      sender: 'user' as const,
      timestamp: new Date('2024-01-01T10:00:00'),
    };

    render(<ChatMessage message={userMessage} />);
    
    const messageElement = screen.getByTestId('chat-message');
    const contentElement = screen.getByText('I need to create a workflow');
    
    expect(messageElement).toBeInTheDocument();
    expect(messageElement).toHaveClass('chat-message--user');
    expect(contentElement).toBeInTheDocument();
  });

  // Test 1b: Renders assistant message with correct styling
  test('renders assistant message with correct styling', () => {
    const assistantMessage = {
      id: '2',
      content: 'I can help you create a workflow. What type would you like?',
      sender: 'assistant' as const,
      timestamp: new Date('2024-01-01T10:01:00'),
    };

    render(<ChatMessage message={assistantMessage} />);
    
    const messageElement = screen.getByTestId('chat-message');
    const contentElement = screen.getByText(/I can help you create a workflow/);
    
    expect(messageElement).toBeInTheDocument();
    expect(messageElement).toHaveClass('chat-message--assistant');
    expect(contentElement).toBeInTheDocument();
  });

  // Test 1c: Displays timestamp
  test('displays formatted timestamp', () => {
    const message = {
      id: '3',
      content: 'Test message',
      sender: 'user' as const,
      timestamp: new Date('2024-01-01T10:30:00'),
    };

    render(<ChatMessage message={message} />);
    
    // Look for time in format like "10:30 AM"
    const timestampElement = screen.getByText(/10:30/);
    expect(timestampElement).toBeInTheDocument();
  });

  // Test 1d: Shows avatar/icon for message sender
  test('shows avatar for user messages', () => {
    const message = {
      id: '4',
      content: 'User message',
      sender: 'user' as const,
      timestamp: new Date(),
    };

    render(<ChatMessage message={message} />);
    
    const avatarElement = screen.getByTestId('user-avatar');
    expect(avatarElement).toBeInTheDocument();
  });

  test('shows avatar for assistant messages', () => {
    const message = {
      id: '5',
      content: 'Assistant message',
      sender: 'assistant' as const,
      timestamp: new Date(),
    };

    render(<ChatMessage message={message} />);
    
    const avatarElement = screen.getByTestId('assistant-avatar');
    expect(avatarElement).toBeInTheDocument();
  });

  // Test 1e: Handles markdown content rendering
  test('renders markdown content correctly', () => {
    const message = {
      id: '6',
      content: '**Bold text** and *italic text* with `code`',
      sender: 'assistant' as const,
      timestamp: new Date(),
    };

    render(<ChatMessage message={message} />);
    
    // Check for rendered markdown elements
    const boldElement = screen.getByText('Bold text');
    const italicElement = screen.getByText('italic text');
    const codeElement = screen.getByText('code');
    
    expect(boldElement).toHaveStyle('font-weight: bold');
    expect(italicElement).toHaveStyle('font-style: italic');
    expect(codeElement.tagName).toBe('CODE');
  });

  test('renders code blocks with syntax highlighting', () => {
    const message = {
      id: '7',
      content: '```typescript\nconst workflow = createWorkflow();\n```',
      sender: 'assistant' as const,
      timestamp: new Date(),
    };

    render(<ChatMessage message={message} />);
    
    const codeBlock = screen.getByTestId('code-block');
    expect(codeBlock).toBeInTheDocument();
    expect(codeBlock).toHaveClass('language-typescript');
  });
});