import { GraphQLClient } from './GraphQLClient';

// RED Phase: Write failing tests first
// These tests will fail initially because GraphQLClient doesn't exist yet

describe('GraphQLClient', () => {
  let client: GraphQLClient;

  beforeEach(() => {
    // Clear any previous instances
    jest.clearAllMocks();
  });

  describe('Connection to GraphQL Gateway', () => {
    it('should connect to GraphQL Gateway on port 4000', () => {
      // RED: This test will fail because GraphQLClient doesn't exist
      client = new GraphQLClient();
      
      expect(client.endpoint).toBe('http://localhost:4000/graphql');
      expect(client.isConnected()).toBe(false); // Initially not connected
    });

    it('should have configurable endpoint', () => {
      // RED: Test for configurable endpoint
      const customEndpoint = 'http://localhost:3000/graphql';
      client = new GraphQLClient(customEndpoint);
      
      expect(client.endpoint).toBe(customEndpoint);
    });

    it('should establish connection when initialized', async () => {
      // RED: Test connection establishment
      client = new GraphQLClient();
      
      await client.connect();
      
      expect(client.isConnected()).toBe(true);
    });
  });

  describe('JWT Authentication', () => {
    it('should authenticate with JWT tokens', async () => {
      // RED: Test JWT authentication
      const token = 'test-jwt-token-123';
      client = new GraphQLClient();
      
      client.setAuthToken(token);
      
      expect(client.getAuthToken()).toBe(token);
    });

    it('should include auth token in requests', async () => {
      // RED: Test that auth token is included in requests
      const token = 'bearer-token-456';
      client = new GraphQLClient();
      client.setAuthToken(token);
      
      const mockQuery = 'query { health }';
      
      // Mock the underlying request method
      const requestSpy = jest.spyOn(client, 'request').mockResolvedValue({ data: { health: 'ok' } });
      
      await client.query(mockQuery);
      
      expect(requestSpy).toHaveBeenCalledWith(
        mockQuery,
        {},
        expect.objectContaining({
          authorization: `Bearer ${token}`
        })
      );
    });

    it('should clear auth token when logout is called', () => {
      // RED: Test auth token clearing
      client = new GraphQLClient();
      client.setAuthToken('some-token');
      
      client.clearAuth();
      
      expect(client.getAuthToken()).toBeNull();
    });
  });

  describe('Error Handling', () => {
    it('should handle GraphQL errors gracefully', async () => {
      // RED: Test GraphQL error handling
      client = new GraphQLClient();
      
      const mockError = {
        errors: [{ message: 'Field not found' }],
        data: null
      };
      
      jest.spyOn(client, 'request').mockRejectedValue(mockError);
      
      await expect(client.query('invalid query')).rejects.toMatchObject({
        errors: expect.arrayContaining([
          expect.objectContaining({ message: 'Field not found' })
        ])
      });
    });

    it('should handle network errors gracefully', async () => {
      // RED: Test network error handling
      client = new GraphQLClient();
      
      const networkError = new Error('Network error');
      jest.spyOn(client, 'request').mockRejectedValue(networkError);
      
      await expect(client.query('query { health }')).rejects.toThrow('Network error');
    });

    it('should provide meaningful error messages', async () => {
      // RED: Test error message formatting
      client = new GraphQLClient();
      
      const graphqlError = {
        errors: [{ message: 'Unauthorized' }],
        data: null
      };
      
      jest.spyOn(client, 'request').mockRejectedValue(graphqlError);
      
      try {
        await client.query('query { workflows }');
      } catch (error: any) {
        expect(error.message).toContain('GraphQL Error: Unauthorized');
      }
    });
  });

  describe('Request Retry Logic', () => {
    it('should retry failed requests', async () => {
      // RED: Test retry functionality
      client = new GraphQLClient();
      
      const requestSpy = jest.spyOn(client, 'request')
        .mockRejectedValueOnce(new Error('Temporary failure'))
        .mockResolvedValueOnce({ data: { health: 'ok' } });
      
      const result = await client.queryWithRetry('query { health }');
      
      expect(requestSpy).toHaveBeenCalledTimes(2);
      expect(result.data.health).toBe('ok');
    });

    it('should limit retry attempts', async () => {
      // RED: Test retry limit
      client = new GraphQLClient();
      
      const requestSpy = jest.spyOn(client, 'request')
        .mockRejectedValue(new Error('Persistent failure'));
      
      await expect(client.queryWithRetry('query { health }', {}, { maxRetries: 3 }))
        .rejects.toThrow('Persistent failure');
      
      expect(requestSpy).toHaveBeenCalledTimes(4); // Initial + 3 retries
    });

    it('should have exponential backoff for retries', async () => {
      // RED: Test exponential backoff
      client = new GraphQLClient();
      
      const startTime = Date.now();
      const requestSpy = jest.spyOn(client, 'request')
        .mockRejectedValue(new Error('Temporary failure'));
      
      try {
        await client.queryWithRetry('query { health }', {}, { maxRetries: 2 });
      } catch (error) {
        // Should have taken some time due to backoff
        const elapsed = Date.now() - startTime;
        expect(elapsed).toBeGreaterThan(100); // At least 100ms for backoff
      }
    });
  });

  describe('Query Operations', () => {
    it('should execute GraphQL queries', async () => {
      // RED: Test basic query execution
      client = new GraphQLClient();
      
      const mockResponse = {
        data: {
          workflows: [
            { id: '1', name: 'Test Workflow', status: 'active' }
          ]
        }
      };
      
      jest.spyOn(client, 'request').mockResolvedValue(mockResponse);
      
      const result = await client.query('query { workflows { id name status } }');
      
      expect(result.data.workflows).toHaveLength(1);
      expect(result.data.workflows[0].name).toBe('Test Workflow');
    });

    it('should execute GraphQL mutations', async () => {
      // RED: Test mutation execution
      client = new GraphQLClient();
      
      const mockResponse = {
        data: {
          createWorkflow: {
            id: '123',
            name: 'New Workflow',
            status: 'created'
          }
        }
      };
      
      jest.spyOn(client, 'request').mockResolvedValue(mockResponse);
      
      const variables = { name: 'New Workflow', description: 'Test workflow' };
      const result = await client.mutate(
        'mutation CreateWorkflow($name: String!, $description: String) { createWorkflow(input: { name: $name, description: $description }) { id name status } }',
        variables
      );
      
      expect(result.data.createWorkflow.name).toBe('New Workflow');
    });

    it('should handle query variables correctly', async () => {
      // RED: Test variable handling
      client = new GraphQLClient();
      
      const requestSpy = jest.spyOn(client, 'request').mockResolvedValue({
        data: { workflow: { id: '1', name: 'Test' } }
      });
      
      const variables = { id: '1' };
      await client.query('query GetWorkflow($id: ID!) { workflow(id: $id) { id name } }', variables);
      
      expect(requestSpy).toHaveBeenCalledWith(
        'query GetWorkflow($id: ID!) { workflow(id: $id) { id name } }',
        variables,
        expect.any(Object)
      );
    });
  });

  describe('Subscription Support', () => {
    it('should support GraphQL subscriptions', async () => {
      // RED: Test subscription support (for future real-time updates)
      client = new GraphQLClient();
      
      const mockSubscription = {
        subscribe: jest.fn(),
        unsubscribe: jest.fn()
      };
      
      jest.spyOn(client, 'subscribe').mockReturnValue(mockSubscription as any);
      
      const subscription = client.subscribe('subscription { workflowStatusChanged(id: "123") }');
      
      expect(subscription).toBeDefined();
      expect(client.subscribe).toHaveBeenCalled();
    });
  });
});