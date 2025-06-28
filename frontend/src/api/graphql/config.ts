// REFACTOR Phase: Extract configuration (Tidy First)
// Separating configuration from behavior

export const GRAPHQL_CONFIG = {
  DEFAULT_ENDPOINT: 'http://localhost:4000/graphql',
  DEFAULT_RETRY_OPTIONS: {
    maxRetries: 3,
    baseDelay: 100
  },
  DEFAULT_TIMEOUT: 30000
} as const;

export interface GraphQLClientConfig {
  endpoint?: string;
  timeout?: number;
  retryOptions?: {
    maxRetries?: number;
    baseDelay?: number;
  };
}