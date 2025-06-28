import { GraphQLClient } from './GraphQLClient';
import { GraphQLResponse, GraphQLError } from './types';

export interface FederationError extends GraphQLError {
  extensions?: {
    code?: string;
    service?: string;
    [key: string]: any;
  };
}

export interface FederationResponse<T = any> {
  data: T;
  errors?: FederationError[];
}

export interface SubscriptionHandler {
  next: (data: any) => void;
  error: (error: Error) => void;
  complete: () => void;
}

export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  services: {
    [serviceName: string]: {
      status: 'up' | 'down';
      latency?: number;
      error?: string;
    };
  };
}

export class FederationClient extends GraphQLClient {
  constructor(gatewayEndpoint: string = 'http://localhost:4000/graphql') {
    super(gatewayEndpoint);
  }

  async query<T = any>(
    query: string,
    variables: Record<string, any> = {}
  ): Promise<FederationResponse<T>> {
    try {
      const response = await super.query(query, variables);
      
      // Transform response to handle federation-specific error format
      return {
        data: response.data,
        errors: response.errors as FederationError[]
      };
    } catch (error: any) {
      // If the error has the GraphQL structure, return it
      if (error.data !== undefined && error.errors) {
        return {
          data: error.data,
          errors: error.errors as FederationError[]
        };
      }
      // Otherwise, throw the error
      throw error;
    }
  }

  async checkHealth(): Promise<HealthStatus> {
    const response = await fetch(`${this.endpoint.replace('/graphql', '/health')}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json'
      }
    });

    if (!response.ok) {
      throw new Error(`Health check failed: ${response.statusText}`);
    }

    return await response.json();
  }

  async subscribeFederated(
    subscription: string,
    variables: Record<string, any> = {},
    handler: SubscriptionHandler
  ): Promise<() => void> {
    // WebSocket URL for subscriptions
    const wsUrl = this.endpoint.replace('http://', 'ws://').replace('https://', 'wss://');
    
    const ws = new WebSocket(wsUrl);
    
    ws.addEventListener('open', () => {
      // Send subscription
      ws.send(JSON.stringify({
        type: 'subscribe',
        payload: {
          query: subscription,
          variables
        }
      }));
    });

    ws.addEventListener('message', (event) => {
      try {
        const message = JSON.parse(event.data);
        
        switch (message.type) {
          case 'data':
            handler.next(message.payload.data);
            break;
          case 'error':
            handler.error(new Error(message.payload.message));
            break;
          case 'complete':
            handler.complete();
            ws.close();
            break;
        }
      } catch (error) {
        handler.error(error as Error);
      }
    });

    ws.addEventListener('error', (error) => {
      handler.error(new Error('WebSocket error'));
    });

    ws.addEventListener('close', () => {
      handler.complete();
    });

    // Return unsubscribe function
    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'unsubscribe' }));
        ws.close();
      }
    };
  }
}