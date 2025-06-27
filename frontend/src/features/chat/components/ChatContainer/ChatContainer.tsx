import React, { useEffect, useRef } from 'react';
import type { ChatContainerProps } from '../../types';
import { UI_TEXT, ARIA_LABELS } from '../../constants';
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
    const hasNewMessages = messages.length > previousMessageCountRef.current;
    const shouldScroll = scrollContainerRef.current && hasNewMessages;
    
    if (shouldScroll && scrollContainerRef.current) {
      scrollContainerRef.current.scrollTo({
        top: scrollContainerRef.current.scrollHeight,
        behavior: 'smooth'
      });
    }
    
    previousMessageCountRef.current = messages.length;
  }, [messages.length]);

  const isEmpty = messages.length === 0;

  return (
    <div className="container">
      <div 
        className="messagesContainer"
        role="log"
        aria-label={ARIA_LABELS.CHAT_MESSAGES}
      >
        <div
          ref={scrollContainerRef}
          data-testid="chat-scroll-container"
          className="scrollContainer"
          aria-label={ARIA_LABELS.CHAT_CONVERSATION}
        >
          {isEmpty ? (
            <EmptyState />
          ) : (
            messages.map((message) => (
              <ChatMessage key={message.id} message={message} />
            ))
          )}
          {isLoading && <LoadingIndicator />}
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

const EmptyState: React.FC = () => (
  <div className="emptyState" data-testid="chat-empty-state">
    <h3>{UI_TEXT.EMPTY_STATE.TITLE}</h3>
    <p>{UI_TEXT.EMPTY_STATE.SUBTITLE}</p>
    <p>{UI_TEXT.EMPTY_STATE.HELPER}</p>
  </div>
);

const LoadingIndicator: React.FC = () => (
  <div className="loading" data-testid="chat-loading">
    {UI_TEXT.LOADING_MESSAGE}
  </div>
);

export default ChatContainer;