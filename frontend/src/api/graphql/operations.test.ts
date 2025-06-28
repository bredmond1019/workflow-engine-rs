import { WorkflowOperations } from './operations';
import { GraphQLClient } from './GraphQLClient';

// RED Phase: Write failing tests for backend integration
// These tests will fail initially because WorkflowOperations doesn't exist yet

describe('WorkflowOperations', () => {
  let operations: WorkflowOperations;
  let mockClient: jest.Mocked<GraphQLClient>;

  beforeEach(() => {
    // Create a mock GraphQL client
    mockClient = {
      query: jest.fn(),
      mutate: jest.fn(),
      subscribe: jest.fn(),
      queryWithRetry: jest.fn(),
      setAuthToken: jest.fn(),
      clearAuth: jest.fn(),
      isConnected: jest.fn(),
      connect: jest.fn(),
      getAuthToken: jest.fn(),
      endpoint: 'http://localhost:4000/graphql'
    } as any;

    // RED: This will fail because WorkflowOperations doesn't exist
    operations = new WorkflowOperations(mockClient);
  });

  describe('Workflow Creation', () => {
    it('should create workflow via GraphQL mutation', async () => {
      // RED: Test creating a workflow through GraphQL
      const workflowData = {
        name: 'Test Workflow',
        description: 'A test workflow for customer support',
        type: 'customer_support'
      };

      const expectedResponse = {
        data: {
          createWorkflow: {
            id: 'wf-123',
            name: 'Test Workflow',
            description: 'A test workflow for customer support',
            status: 'created',
            createdAt: '2024-01-01T10:00:00Z'
          }
        }
      };

      mockClient.mutate.mockResolvedValue(expectedResponse);

      const result = await operations.createWorkflow(workflowData);

      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining('mutation CreateWorkflow'),
        expect.objectContaining({
          input: workflowData
        })
      );
      expect(result.id).toBe('wf-123');
      expect(result.name).toBe('Test Workflow');
    });

    it('should handle workflow creation errors', async () => {
      // RED: Test error handling during workflow creation
      const workflowData = {
        name: '',
        description: 'Invalid workflow'
      };

      const errorResponse = {
        errors: [{ message: 'Workflow name is required' }],
        data: null
      };

      mockClient.mutate.mockRejectedValue(errorResponse);

      await expect(operations.createWorkflow(workflowData)).rejects.toMatchObject({
        errors: expect.arrayContaining([
          expect.objectContaining({ message: 'Workflow name is required' })
        ])
      });
    });

    it('should transform chat intent to workflow mutation variables', async () => {
      // RED: Test transformation from chat intent to GraphQL variables
      const chatIntent = {
        type: 'CREATE_WORKFLOW',
        workflowType: 'CUSTOMER_SUPPORT',
        parameters: [
          { name: 'source_service', value: 'helpscout' },
          { name: 'destination_service', value: 'slack' }
        ],
        detectedServices: ['helpscout', 'slack']
      };

      const expectedMutationVars = {
        input: {
          name: 'Customer Support Workflow',
          type: 'customer_support',
          nodes: expect.arrayContaining([
            expect.objectContaining({ type: 'helpscout_source' }),
            expect.objectContaining({ type: 'slack_destination' })
          ])
        }
      };

      mockClient.mutate.mockResolvedValue({
        data: { createWorkflow: { id: 'wf-456' } }
      });

      await operations.createWorkflowFromIntent(chatIntent);

      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining('mutation CreateWorkflow'),
        expectedMutationVars
      );
    });
  });

  describe('Workflow Status Queries', () => {
    it('should fetch workflow status via GraphQL query', async () => {
      // RED: Test fetching workflow status
      const workflowId = 'wf-123';
      const expectedResponse = {
        data: {
          workflow: {
            id: 'wf-123',
            name: 'Test Workflow',
            status: 'running',
            steps: [
              { id: 'step-1', status: 'completed', output: 'Step 1 completed' },
              { id: 'step-2', status: 'running', output: null }
            ],
            executionProgress: 50
          }
        }
      };

      mockClient.query.mockResolvedValue(expectedResponse);

      const result = await operations.getWorkflowStatus(workflowId);

      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining('query GetWorkflowStatus'),
        { id: workflowId }
      );
      expect(result.status).toBe('running');
      expect(result.executionProgress).toBe(50);
      expect(result.steps).toHaveLength(2);
    });

    it('should fetch workflow list for preview', async () => {
      // RED: Test fetching workflow list for WorkflowPreview component
      const expectedResponse = {
        data: {
          workflows: {
            items: [
              { id: 'wf-1', name: 'Workflow 1', status: 'active' },
              { id: 'wf-2', name: 'Workflow 2', status: 'draft' }
            ],
            totalCount: 2
          }
        }
      };

      mockClient.query.mockResolvedValue(expectedResponse);

      const result = await operations.listWorkflows({ limit: 10, offset: 0 });

      expect(mockClient.query).toHaveBeenCalledWith(
        expect.stringContaining('query ListWorkflows'),
        { limit: 10, offset: 0 }
      );
      expect(result.items).toHaveLength(2);
      expect(result.totalCount).toBe(2);
    });

    it('should handle network errors gracefully', async () => {
      // RED: Test network error handling
      const networkError = new Error('Network error');
      mockClient.query.mockRejectedValue(networkError);

      await expect(operations.getWorkflowStatus('wf-123')).rejects.toThrow('Network error');
    });
  });

  describe('Real-time Subscriptions', () => {
    it('should subscribe to workflow status changes', async () => {
      // RED: Test GraphQL subscription for real-time updates
      const workflowId = 'wf-123';
      const mockSubscription = {
        subscribe: jest.fn(),
        unsubscribe: jest.fn()
      };

      mockClient.subscribe.mockReturnValue(mockSubscription);

      const subscription = operations.subscribeToWorkflowUpdates(workflowId, (data) => {
        // Callback for updates
      });

      expect(mockClient.subscribe).toHaveBeenCalledWith(
        expect.stringContaining('subscription WorkflowStatusChanged')
      );
      expect(subscription).toBe(mockSubscription);
    });

    it('should handle subscription errors', async () => {
      // RED: Test subscription error handling
      const workflowId = 'wf-123';
      const subscriptionError = new Error('Subscription failed');
      
      mockClient.subscribe.mockImplementation(() => {
        throw subscriptionError;
      });

      expect(() => operations.subscribeToWorkflowUpdates(workflowId, () => {}))
        .toThrow('Subscription failed');
    });
  });

  describe('Authentication Integration', () => {
    it('should set auth token from auth store', async () => {
      // RED: Test auth token integration
      const token = 'jwt-token-123';
      
      operations.setAuthToken(token);

      expect(mockClient.setAuthToken).toHaveBeenCalledWith(token);
    });

    it('should clear auth on logout', async () => {
      // RED: Test auth clearing
      operations.clearAuth();

      expect(mockClient.clearAuth).toHaveBeenCalled();
    });

    it('should retry operations with auth token refresh', async () => {
      // RED: Test auth token refresh on 401 errors
      const successResponse = {
        data: { workflow: { id: 'wf-123', status: 'active' } }
      };

      // Mock successful response for queryWithRetry
      mockClient.queryWithRetry.mockResolvedValue(successResponse);

      const result = await operations.getWorkflowStatusWithRetry('wf-123');

      expect(mockClient.queryWithRetry).toHaveBeenCalledTimes(1);
      expect(result.id).toBe('wf-123');
    });
  });

  describe('Integration with Frontend Components', () => {
    it('should format workflow data for WorkflowPreview component', async () => {
      // RED: Test data formatting for frontend components
      const rawWorkflowData = {
        id: 'wf-123',
        name: 'Customer Support Workflow',
        description: 'Handles support tickets',
        nodes: [
          { id: 'node-1', type: 'trigger', config: { service: 'helpscout' } },
          { id: 'node-2', type: 'action', config: { service: 'slack' } }
        ],
        connections: [
          { from: 'node-1', to: 'node-2' }
        ]
      };

      mockClient.query.mockResolvedValue({
        data: { workflow: rawWorkflowData }
      });

      const formattedData = await operations.getWorkflowForPreview('wf-123');

      expect(formattedData).toMatchObject({
        id: 'wf-123',
        title: 'Customer Support Workflow',
        description: 'Handles support tickets',
        nodes: expect.arrayContaining([
          expect.objectContaining({
            id: 'node-1',
            type: 'trigger',
            label: expect.any(String)
          })
        ]),
        connections: expect.arrayContaining([
          expect.objectContaining({
            source: 'node-1',
            target: 'node-2'
          })
        ])
      });
    });

    it('should validate form data before workflow creation', async () => {
      // RED: Test form validation integration
      const invalidFormData = {
        name: '',
        description: 'Test',
        // Missing required fields
      };

      const validationErrors = await operations.validateWorkflowForm(invalidFormData);

      expect(validationErrors).toContain('Name is required');
      expect(validationErrors.length).toBeGreaterThan(0);
    });

    it('should transform DynamicForm submission to GraphQL mutation', async () => {
      // RED: Test DynamicForm integration
      const formSubmission = {
        workflowName: 'Support Automation',
        sourceService: 'helpscout',
        destinationService: 'slack',
        triggerCondition: 'high_priority',
        slackChannel: '#support'
      };

      const expectedMutation = {
        input: {
          name: 'Support Automation',
          type: 'customer_support',
          nodes: expect.arrayContaining([
            expect.objectContaining({
              type: 'helpscout_trigger',
              config: { condition: 'high_priority' }
            }),
            expect.objectContaining({
              type: 'slack_notification',
              config: { channel: '#support' }
            })
          ])
        }
      };

      mockClient.mutate.mockResolvedValue({
        data: { createWorkflow: { id: 'wf-789' } }
      });

      await operations.createWorkflowFromForm(formSubmission);

      expect(mockClient.mutate).toHaveBeenCalledWith(
        expect.stringContaining('mutation CreateWorkflow'),
        expectedMutation
      );
    });
  });
});