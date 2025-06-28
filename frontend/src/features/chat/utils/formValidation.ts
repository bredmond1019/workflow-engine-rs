import { FormField } from '../types/workflow';

// Email validation regex
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/**
 * Validates an email address format
 */
export const isValidEmail = (email: string): boolean => {
  return EMAIL_REGEX.test(email);
};

/**
 * Validates a single form field based on its configuration
 */
export const validateField = (field: FormField, value: any): string | null => {
  if (field.required && (!value || value === '')) {
    return `${field.label} is required`;
  }

  if (field.type === 'email' && value && !isValidEmail(value)) {
    return 'Please enter a valid email address';
  }

  if (field.validation) {
    const { min, max, message } = field.validation;
    if (field.type === 'number') {
      const numValue = Number(value);
      if ((min !== undefined && numValue < min) || (max !== undefined && numValue > max)) {
        return message || `Value must be between ${min} and ${max}`;
      }
    }
  }

  return null;
};

/**
 * Calculates form completion statistics
 */
export const getCompletionStats = (
  fields: FormField[], 
  values: Record<string, any>
) => {
  const completedCount = fields.filter(field => 
    values[field.name] !== undefined && values[field.name] !== ''
  ).length;
  
  const totalCount = fields.length;
  const percentage = totalCount > 0 ? Math.round((completedCount / totalCount) * 100) : 0;
  
  return { completedCount, totalCount, percentage };
};

/**
 * Formats step name from snake_case to Title Case
 */
export const formatStepName = (stepName: string): string => {
  return stepName.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
};