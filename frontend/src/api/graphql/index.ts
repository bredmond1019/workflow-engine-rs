// REFACTOR Phase: Clean public API (Tidy First)
// Providing a clean interface for consumers

export { GraphQLClient } from './GraphQLClient';
export { WorkflowOperations } from './operations';
export { FederationClient } from './FederationClient';
export type { 
  GraphQLResponse, 
  RetryOptions, 
  Subscription, 
  GraphQLError 
} from './types';
export type { 
  GraphQLClientConfig 
} from './config';
export type {
  WorkflowData,
  WorkflowStatus,
  WorkflowList,
  WorkflowPreviewData,
  ChatIntent
} from './operations';
export type {
  FederationError,
  FederationResponse,
  SubscriptionHandler,
  HealthStatus
} from './FederationClient';
export { GRAPHQL_CONFIG } from './config';