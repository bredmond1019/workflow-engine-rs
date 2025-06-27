import React from 'react';
import { DEFAULT_CODE_LANGUAGE } from '../constants';

/**
 * Renders markdown content with support for code blocks, inline code, bold, and italic
 */
export const renderMarkdown = (content: string): React.ReactElement => {
  let html = content;
  
  // Process code blocks first
  html = processCodeBlocks(html);
  
  // Process inline formatting
  html = processInlineCode(html);
  html = processBold(html);
  html = processItalic(html);
  
  return <div dangerouslySetInnerHTML={{ __html: html }} />;
};

/**
 * Processes code blocks with optional language specification
 */
const processCodeBlocks = (text: string): string => {
  if (!text.includes('```')) {
    return text;
  }
  
  const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g;
  
  return text.replace(codeBlockRegex, (_match, lang, code) => {
    const language = lang || DEFAULT_CODE_LANGUAGE;
    const trimmedCode = code.trim();
    return `<pre data-testid="code-block" class="language-${language}">${trimmedCode}</pre>`;
  });
};

/**
 * Processes inline code formatting
 */
const processInlineCode = (text: string): string => {
  return text.replace(/`([^`]+)`/g, '<code>$1</code>');
};

/**
 * Processes bold text formatting
 */
const processBold = (text: string): string => {
  return text.replace(/\*\*([^*]+)\*\*/g, '<strong style="font-weight: bold">$1</strong>');
};

/**
 * Processes italic text formatting
 */
const processItalic = (text: string): string => {
  return text.replace(/\*([^*]+)\*/g, '<em style="font-style: italic">$1</em>');
};