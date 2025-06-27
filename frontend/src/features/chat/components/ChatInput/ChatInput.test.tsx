import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import ChatInput from './ChatInput';

describe('ChatInput Component', () => {
  const mockOnSendMessage = jest.fn();

  beforeEach(() => {
    mockOnSendMessage.mockClear();
  });

  // Test 2a: Renders input field with placeholder
  test('renders input field with placeholder', () => {
    render(<ChatInput onSendMessage={mockOnSendMessage} />);
    
    const inputElement = screen.getByRole('textbox');
    expect(inputElement).toBeInTheDocument();
    expect(inputElement).toHaveAttribute('placeholder', 'Type a message...');
  });

  test('renders custom placeholder when provided', () => {
    render(
      <ChatInput 
        onSendMessage={mockOnSendMessage} 
        placeholder="Ask me anything..."
      />
    );
    
    const inputElement = screen.getByRole('textbox');
    expect(inputElement).toHaveAttribute('placeholder', 'Ask me anything...');
  });

  // Test 2b: Handles text input changes
  test('handles text input changes', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} />);
    
    const inputElement = screen.getByRole('textbox');
    await user.type(inputElement, 'Hello, AI assistant!');
    
    expect(inputElement).toHaveValue('Hello, AI assistant!');
  });

  // Test 2c: Submits on Enter key press
  test('submits message on Enter key press', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} />);
    
    const inputElement = screen.getByRole('textbox');
    await user.type(inputElement, 'Test message');
    await user.keyboard('{Enter}');
    
    expect(mockOnSendMessage).toHaveBeenCalledWith('Test message');
    expect(mockOnSendMessage).toHaveBeenCalledTimes(1);
  });

  test('does not submit empty message', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} />);
    
    screen.getByRole('textbox');
    await user.keyboard('{Enter}');
    
    expect(mockOnSendMessage).not.toHaveBeenCalled();
  });

  // Test 2d: Clears input after submission
  test('clears input after successful submission', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} />);
    
    const inputElement = screen.getByRole('textbox');
    await user.type(inputElement, 'Message to send');
    await user.keyboard('{Enter}');
    
    expect(inputElement).toHaveValue('');
  });

  // Test 2e: Disables input while processing
  test('disables input when disabled prop is true', () => {
    render(<ChatInput onSendMessage={mockOnSendMessage} disabled={true} />);
    
    const inputElement = screen.getByRole('textbox');
    expect(inputElement).toBeDisabled();
  });

  test('shows loading state when disabled', () => {
    render(<ChatInput onSendMessage={mockOnSendMessage} disabled={true} />);
    
    const sendButton = screen.getByTestId('send-button');
    expect(sendButton).toBeDisabled();
    expect(screen.getByTestId('loading-indicator')).toBeInTheDocument();
  });

  // Test 2f: Shows character count limit
  test('shows character count when approaching limit', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} maxLength={100} />);
    
    const inputElement = screen.getByRole('textbox');
    const longText = 'a'.repeat(80); // 80 characters
    await user.type(inputElement, longText);
    
    const charCount = screen.getByTestId('character-count');
    expect(charCount).toBeInTheDocument();
    expect(charCount).toHaveTextContent('80 / 100');
  });

  test('prevents input beyond max length', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} maxLength={10} />);
    
    const inputElement = screen.getByRole('textbox');
    await user.type(inputElement, 'This is a very long message');
    
    expect((inputElement as HTMLTextAreaElement).value).toHaveLength(10);
  });

  test('shows warning color when near limit', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} maxLength={100} />);
    
    const inputElement = screen.getByRole('textbox');
    const longText = 'a'.repeat(95); // 95% of limit
    await user.type(inputElement, longText);
    
    const charCount = screen.getByTestId('character-count');
    expect(charCount).toHaveClass('character-count--warning');
  });

  // Additional tests for send button
  test('sends message when send button is clicked', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} />);
    
    const inputElement = screen.getByRole('textbox');
    await user.type(inputElement, 'Click to send');
    
    const sendButton = screen.getByTestId('send-button');
    await user.click(sendButton);
    
    expect(mockOnSendMessage).toHaveBeenCalledWith('Click to send');
  });

  test('handles multiline input with Shift+Enter', async () => {
    const user = userEvent.setup();
    render(<ChatInput onSendMessage={mockOnSendMessage} />);
    
    const inputElement = screen.getByRole('textbox');
    await user.type(inputElement, 'Line 1');
    await user.keyboard('{Shift>}{Enter}{/Shift}');
    await user.type(inputElement, 'Line 2');
    
    expect(inputElement).toHaveValue('Line 1\nLine 2');
    expect(mockOnSendMessage).not.toHaveBeenCalled();
  });
});