import React from 'react';
import type { ChatMessage as ChatMessageType } from '../../types';
import { CSS_CLASSES } from '../../constants';
import { formatTime, buildClassName } from '../../utils';
import { renderMarkdown } from '../../utils/markdown';
import Avatar from '../Avatar';
import './ChatMessage.module.css';

export interface ChatMessageProps {
  message: ChatMessageType;
}

const ChatMessage: React.FC<ChatMessageProps> = ({ message }) => {
  const messageClassName = buildClassName(CSS_CLASSES.CHAT_MESSAGE, message.sender);

  return (
    <div 
      data-testid="chat-message"
      className={messageClassName}
    >
      <div className="chat-message__avatar">
        <Avatar type={message.sender === 'user' ? 'user' : 'assistant'} />
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