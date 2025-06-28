// REFACTOR Phase: Clean public API (Tidy First)
// Providing a clean interface for consumers

export { GraphQLClient } from './GraphQLClient';
export type { GraphQLResponse, RetryOptions, Subscription, GraphQLError } from './types';
export type { GraphQLClientConfig } from './config';
export { GRAPHQL_CONFIG } from './config';