import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import ChatContainer from './ChatContainer';
import { ChatMessage } from '../../types';

describe('ChatContainer Component', () => {
  const mockMessages: ChatMessage[] = [
    {
      id: '1',
      content: 'Hello, how can I help you?',
      sender: 'assistant',
      timestamp: new Date('2024-01-01T10:00:00'),
    },
    {
      id: '2',
      content: 'I need help creating a workflow',
      sender: 'user',
      timestamp: new Date('2024-01-01T10:01:00'),
    },
    {
      id: '3',
      content: 'I can help you with that. What type of workflow?',
      sender: 'assistant',
      timestamp: new Date('2024-01-01T10:02:00'),
    },
  ];

  const mockOnSendMessage = jest.fn();

  beforeEach(() => {
    mockOnSendMessage.mockClear();
  });

  // Test 3a: Renders list of messages in order
  test('renders list of messages in chronological order', () => {
    render(
      <ChatContainer 
        messages={mockMessages} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    const messageElements = screen.getAllByTestId('chat-message');
    expect(messageElements).toHaveLength(3);
    
    // Check messages are in order
    expect(messageElements[0]).toHaveTextContent('Hello, how can I help you?');
    expect(messageElements[1]).toHaveTextContent('I need help creating a workflow');
    expect(messageElements[2]).toHaveTextContent('What type of workflow?');
  });

  // Test 3b: Auto-scrolls to bottom on new message
  test('auto-scrolls to bottom when new message is added', async () => {
    const { rerender } = render(
      <ChatContainer 
        messages={mockMessages} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    const scrollContainer = screen.getByTestId('chat-scroll-container');
    const scrollToSpy = jest.spyOn(scrollContainer, 'scrollTo');
    
    // Add a new message
    const newMessage: ChatMessage = {
      id: '4',
      content: 'A customer support workflow',
      sender: 'user',
      timestamp: new Date('2024-01-01T10:03:00'),
    };
    
    rerender(
      <ChatContainer 
        messages={[...mockMessages, newMessage]} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    await waitFor(() => {
      expect(scrollToSpy).toHaveBeenCalledWith({
        top: scrollContainer.scrollHeight,
        behavior: 'smooth',
      });
    });
  });

  // Test 3c: Maintains scroll position when loading history
  test('maintains scroll position when loading message history', async () => {
    const { rerender } = render(
      <ChatContainer 
        messages={mockMessages} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    const scrollContainer = screen.getByTestId('chat-scroll-container');
    
    // Set scroll position to middle
    Object.defineProperty(scrollContainer, 'scrollTop', {
      writable: true,
      value: 100,
    });
    
    // Prepend older messages (loading history)
    const olderMessages: ChatMessage[] = [
      {
        id: '0',
        content: 'Previous conversation',
        sender: 'user',
        timestamp: new Date('2024-01-01T09:59:00'),
      },
      ...mockMessages,
    ];
    
    rerender(
      <ChatContainer 
        messages={olderMessages} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    // Scroll position should be maintained (not jump to bottom)
    expect(scrollContainer.scrollTop).toBeGreaterThan(0);
    expect(scrollContainer.scrollTop).not.toBe(scrollContainer.scrollHeight);
  });

  // Test 3d: Shows loading indicator when fetching
  test('shows loading indicator when isLoading is true', () => {
    render(
      <ChatContainer 
        messages={[]} 
        onSendMessage={mockOnSendMessage}
        isLoading={true}
      />
    );
    
    const loadingIndicator = screen.getByTestId('chat-loading');
    expect(loadingIndicator).toBeInTheDocument();
    expect(loadingIndicator).toHaveTextContent('AI is thinking...');
  });

  test('hides loading indicator when isLoading is false', () => {
    render(
      <ChatContainer 
        messages={mockMessages} 
        onSendMessage={mockOnSendMessage}
        isLoading={false}
      />
    );
    
    expect(screen.queryByTestId('chat-loading')).not.toBeInTheDocument();
  });

  // Test 3e: Displays empty state when no messages
  test('displays empty state when messages array is empty', () => {
    render(
      <ChatContainer 
        messages={[]} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    const emptyState = screen.getByTestId('chat-empty-state');
    expect(emptyState).toBeInTheDocument();
    expect(emptyState).toHaveTextContent('Start a conversation');
  });

  test('shows welcome message in empty state', () => {
    render(
      <ChatContainer 
        messages={[]} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    expect(screen.getByText(/I'm here to help you create workflows/)).toBeInTheDocument();
    expect(screen.getByText(/Type a message below to get started/)).toBeInTheDocument();
  });

  // Integration tests
  test('integrates ChatInput component for sending messages', async () => {
    const user = userEvent.setup();
    
    render(
      <ChatContainer 
        messages={mockMessages} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    const inputElement = screen.getByRole('textbox');
    expect(inputElement).toBeInTheDocument();
    
    await user.type(inputElement, 'New message');
    await user.keyboard('{Enter}');
    
    expect(mockOnSendMessage).toHaveBeenCalledWith('New message');
  });

  test('disables input while loading', () => {
    render(
      <ChatContainer 
        messages={mockMessages} 
        onSendMessage={mockOnSendMessage}
        isLoading={true}
      />
    );
    
    const inputElement = screen.getByRole('textbox');
    expect(inputElement).toBeDisabled();
  });

  // Accessibility tests
  test('has proper ARIA labels for screen readers', () => {
    render(
      <ChatContainer 
        messages={mockMessages} 
        onSendMessage={mockOnSendMessage}
      />
    );
    
    const chatRegion = screen.getByRole('log', { name: 'Chat messages' });
    expect(chatRegion).toBeInTheDocument();
    
    const messagesContainer = screen.getByLabelText('Chat conversation');
    expect(messagesContainer).toBeInTheDocument();
  });
});