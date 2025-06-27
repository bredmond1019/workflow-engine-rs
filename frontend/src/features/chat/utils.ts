// Chat feature utility functions

import { TIME_FORMAT_OPTIONS } from './constants';

/**
 * Formats a Date object to a human-readable time string
 */
export const formatTime = (date: Date): string => {
  return new Intl.DateTimeFormat('en-US', TIME_FORMAT_OPTIONS).format(date);
};

/**
 * Builds CSS class names with optional modifiers
 */
export const buildClassName = (
  baseClass: string,
  modifier?: string,
  additionalClasses?: string
): string => {
  const classes = [baseClass];
  
  if (modifier) {
    classes.push(`${baseClass}--${modifier}`);
  }
  
  if (additionalClasses) {
    classes.push(additionalClasses);
  }
  
  return classes.join(' ');
};

/**
 * Determines if character count should be displayed
 */
export const shouldShowCharCount = (
  currentLength: number,
  maxLength: number | undefined,
  threshold: number
): boolean => {
  return Boolean(maxLength && currentLength > maxLength * threshold);
};

/**
 * Checks if text content is empty or only contains whitespace
 */
export const isEmptyMessage = (message: string): boolean => {
  return !message.trim();
};