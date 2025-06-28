// REFACTOR Phase: Extract types (Tidy First)
// Separating type definitions for better organization

export interface GraphQLResponse<T = any> {
  data: T;
  errors?: Array<{ message: string }>;
}

export interface RetryOptions {
  maxRetries?: number;
  baseDelay?: number;
}

export interface Subscription {
  subscribe: () => void;
  unsubscribe: () => void;
}

export interface GraphQLError {
  message: string;
  locations?: Array<{
    line: number;
    column: number;
  }>;
  path?: Array<string | number>;
  extensions?: Record<string, any>;
}