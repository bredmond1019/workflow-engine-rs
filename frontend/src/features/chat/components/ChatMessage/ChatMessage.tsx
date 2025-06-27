import React from 'react';
import { ChatMessage as ChatMessageType } from '../../types';
import './ChatMessage.module.css';

interface ChatMessageProps {
  message: ChatMessageType;
}

const ChatMessage: React.FC<ChatMessageProps> = ({ message }) => {
  const formatTime = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    }).format(date);
  };

  const renderMarkdown = (content: string) => {
    // Simple markdown rendering - just handle bold, italic, and code
    let html = content;
    
    // Code blocks
    if (content.includes('```')) {
      const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g;
      html = html.replace(codeBlockRegex, (match, lang, code) => {
        return `<pre data-testid="code-block" class="language-${lang || 'plaintext'}">${code.trim()}</pre>`;
      });
    }
    
    // Inline code
    html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
    
    // Bold
    html = html.replace(/\*\*([^*]+)\*\*/g, '<strong style="font-weight: bold">$1</strong>');
    
    // Italic
    html = html.replace(/\*([^*]+)\*/g, '<em style="font-style: italic">$1</em>');
    
    return <div dangerouslySetInnerHTML={{ __html: html }} />;
  };

  return (
    <div 
      data-testid="chat-message"
      className={`chat-message chat-message--${message.sender}`}
    >
      <div className="chat-message__avatar">
        {message.sender === 'user' ? (
          <div data-testid="user-avatar" className="avatar avatar--user">
            U
          </div>
        ) : (
          <div data-testid="assistant-avatar" className="avatar avatar--assistant">
            AI
          </div>
        )}
      </div>
      <div className="chat-message__content">
        <div className="chat-message__bubble">
          {renderMarkdown(message.content)}
        </div>
        <div className="chat-message__timestamp">
          {formatTime(message.timestamp)}
        </div>
      </div>
    </div>
  );
};

export default ChatMessage;