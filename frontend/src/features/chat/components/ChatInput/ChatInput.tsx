import React, { useState } from 'react';
import type { KeyboardEvent, ChangeEvent } from 'react';
import type { ChatInputProps } from '../../types';
import { CHARACTER_COUNT, UI_TEXT, CSS_CLASSES, ICONS } from '../../constants';
import { shouldShowCharCount, isEmptyMessage, buildClassName } from '../../utils';
import './ChatInput.module.css';

const ChatInput: React.FC<ChatInputProps> = ({
  onSendMessage,
  disabled = false,
  placeholder = UI_TEXT.DEFAULT_PLACEHOLDER,
  maxLength,
}) => {
  const [message, setMessage] = useState('');

  const handleSend = () => {
    if (!isEmptyMessage(message) && !disabled) {
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

  const showCharCount = shouldShowCharCount(message.length, maxLength, CHARACTER_COUNT.SHOW_THRESHOLD);
  const isNearLimit = shouldShowCharCount(message.length, maxLength, CHARACTER_COUNT.WARNING_THRESHOLD);
  const charCountClassName = buildClassName(
    CSS_CLASSES.CHARACTER_COUNT,
    isNearLimit ? 'warning' : undefined
  );

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
            className={charCountClassName}
          >
            {message.length} / {maxLength}
          </div>
        )}
      </div>
      
      <button
        data-testid="send-button"
        className="chat-input__send"
        onClick={handleSend}
        disabled={disabled || isEmptyMessage(message)}
        type="button"
      >
        {disabled ? (
          <span data-testid="loading-indicator" className="loading-indicator">
            {ICONS.LOADING}
          </span>
        ) : (
          <span className="send-icon">{ICONS.SEND}</span>
        )}
      </button>
    </div>
  );
};

export default ChatInput;