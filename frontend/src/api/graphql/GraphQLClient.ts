// REFACTOR Phase: Improved structure using extracted types and config
// Following Kent Beck's "Tidy First" principles

import { GraphQLResponse, RetryOptions, Subscription } from './types';
import { GRAPHQL_CONFIG, GraphQLClientConfig } from './config';

export class GraphQLClient {
  public endpoint: string;
  private authToken: string | null = null;
  private connected: boolean = false;
  private config: GraphQLClientConfig;

  constructor(endpoint?: string, config?: GraphQLClientConfig) {
    this.endpoint = endpoint || GRAPHQL_CONFIG.DEFAULT_ENDPOINT;
    this.config = {
      timeout: GRAPHQL_CONFIG.DEFAULT_TIMEOUT,
      retryOptions: GRAPHQL_CONFIG.DEFAULT_RETRY_OPTIONS,
      ...config
    };
  }

  isConnected(): boolean {
    return this.connected;
  }

  async connect(): Promise<void> {
    // Minimal implementation - just set connected flag
    this.connected = true;
  }

  setAuthToken(token: string): void {
    this.authToken = token;
  }

  getAuthToken(): string | null {
    return this.authToken;
  }

  clearAuth(): void {
    this.authToken = null;
  }

  async request(query: string, variables: Record<string, any> = {}, headers: Record<string, string> = {}): Promise<GraphQLResponse> {
    // Minimal implementation - this would be mocked in tests
    throw new Error('Not implemented - should be mocked in tests');
  }

  private formatGraphQLError(error: any): Error {
    // REFACTOR: Extract error handling logic (Tidy First)
    if (error.errors) {
      // Create error with meaningful message while preserving structure
      const meaningfulError = new Error(`GraphQL Error: ${error.errors[0].message}`);
      // Preserve original errors array for matching
      (meaningfulError as any).errors = error.errors;
      (meaningfulError as any).data = error.data;
      return meaningfulError;
    }
    return error;
  }

  private buildHeaders(): Record<string, string> {
    // REFACTOR: Extract header building logic (Tidy First)
    const headers: Record<string, string> = {};
    
    if (this.authToken) {
      headers.authorization = `Bearer ${this.authToken}`;
    }

    return headers;
  }

  async query(query: string, variables: Record<string, any> = {}): Promise<GraphQLResponse> {
    const headers = this.buildHeaders();

    try {
      return await this.request(query, variables, headers);
    } catch (error: any) {
      throw this.formatGraphQLError(error);
    }
  }

  async mutate(mutation: string, variables: Record<string, any> = {}): Promise<GraphQLResponse> {
    // Mutations are just queries, so delegate to query method
    return this.query(mutation, variables);
  }

  private async delay(ms: number): Promise<void> {
    // REFACTOR: Extract delay utility (Tidy First)
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private calculateBackoffDelay(attempt: number, baseDelay: number): number {
    // REFACTOR: Extract backoff calculation (Tidy First)
    return baseDelay * Math.pow(2, attempt);
  }

  async queryWithRetry(
    query: string, 
    variables: Record<string, any> = {}, 
    options: RetryOptions = {}
  ): Promise<GraphQLResponse> {
    // Use config defaults merged with provided options
    const retryConfig = { ...this.config.retryOptions, ...options };
    const { maxRetries = 3, baseDelay = 100 } = retryConfig;
    let lastError: Error;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await this.query(query, variables);
      } catch (error: any) {
        lastError = error;
        
        if (attempt < maxRetries) {
          const delay = this.calculateBackoffDelay(attempt, baseDelay);
          await this.delay(delay);
        }
      }
    }

    throw lastError!;
  }

  subscribe(subscription: string): Subscription {
    // Minimal implementation for subscription support
    return {
      subscribe: () => {
        // Placeholder
      },
      unsubscribe: () => {
        // Placeholder  
      }
    };
  }
}