import React, { useEffect, useRef } from 'react';
import { ChatContainerProps } from '../../types';
import ChatMessage from '../ChatMessage';
import ChatInput from '../ChatInput';
import './ChatContainer.module.css';

const ChatContainer: React.FC<ChatContainerProps> = ({ 
  messages, 
  isLoading = false, 
  onSendMessage 
}) => {
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const previousMessageCountRef = useRef(messages.length);

  useEffect(() => {
    // Auto-scroll to bottom on new messages (not when loading history)
    if (scrollContainerRef.current && messages.length > previousMessageCountRef.current) {
      scrollContainerRef.current.scrollTo({
        top: scrollContainerRef.current.scrollHeight,
        behavior: 'smooth'
      });
    }
    previousMessageCountRef.current = messages.length;
  }, [messages.length]);

  return (
    <div className="container">
      <div 
        className="messagesContainer"
        role="log"
        aria-label="Chat messages"
      >
        <div
          ref={scrollContainerRef}
          data-testid="chat-scroll-container"
          className="scrollContainer"
          aria-label="Chat conversation"
        >
          {messages.length === 0 ? (
            <div className="emptyState" data-testid="chat-empty-state">
              <h3>Start a conversation</h3>
              <p>I'm here to help you create workflows</p>
              <p>Type a message below to get started</p>
            </div>
          ) : (
            messages.map((message) => (
              <ChatMessage key={message.id} message={message} />
            ))
          )}
          {isLoading && (
            <div className="loading" data-testid="chat-loading">
              AI is thinking...
            </div>
          )}
        </div>
      </div>
      <div className="inputContainer">
        <ChatInput 
          onSendMessage={onSendMessage} 
          disabled={isLoading}
        />
      </div>
    </div>
  );
};

export default ChatContainer;