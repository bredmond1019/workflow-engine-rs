import React, { useState, KeyboardEvent, ChangeEvent } from 'react';
import { ChatInputProps } from '../../types';
import './ChatInput.module.css';

const ChatInput: React.FC<ChatInputProps> = ({
  onSendMessage,
  disabled = false,
  placeholder = 'Type a message...',
  maxLength,
}) => {
  const [message, setMessage] = useState('');

  const handleSend = () => {
    if (message.trim() && !disabled) {
      onSendMessage(message);
      setMessage('');
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleChange = (e: ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    if (!maxLength || value.length <= maxLength) {
      setMessage(value);
    }
  };

  const showCharCount = maxLength && message.length > maxLength * 0.7;
  const isNearLimit = maxLength && message.length > maxLength * 0.9;

  return (
    <div className="chat-input">
      <div className="chat-input__wrapper">
        <textarea
          className="chat-input__field"
          value={message}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled}
          rows={1}
          maxLength={maxLength}
        />
        
        {showCharCount && (
          <div 
            data-testid="character-count"
            className={`character-count ${isNearLimit ? 'character-count--warning' : ''}`}
          >
            {message.length} / {maxLength}
          </div>
        )}
      </div>
      
      <button
        data-testid="send-button"
        className="chat-input__send"
        onClick={handleSend}
        disabled={disabled || !message.trim()}
        type="button"
      >
        {disabled ? (
          <span data-testid="loading-indicator" className="loading-indicator">
            ⋯
          </span>
        ) : (
          <span className="send-icon">➤</span>
        )}
      </button>
    </div>
  );
};

export default ChatInput;