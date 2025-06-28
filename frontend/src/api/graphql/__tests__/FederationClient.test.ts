import { GraphQLClient } from '../GraphQLClient';
import { FederationClient } from '../FederationClient';

describe('FederationClient', () => {
  let federationClient: FederationClient;
  let mockFetch: jest.Mock;

  beforeEach(() => {
    mockFetch = jest.fn();
    (globalThis as any).fetch = mockFetch;
    federationClient = new FederationClient('http://localhost:4000/graphql');
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  describe('Federated Query Execution', () => {
    it('should execute cross-service queries through gateway', async () => {
      const federatedQuery = `
        query GetWorkflowWithContent($workflowId: ID!) {
          workflow(id: $workflowId) {
            id
            name
            # From workflow-api service
            nodes {
              id
              type
            }
            # From content-processing service via federation
            processedContent {
              id
              summary
              extractedEntities
            }
            # From knowledge-graph service via federation
            relatedKnowledge {
              concepts
              relationships
            }
          }
        }
      `;

      // Mock the GraphQLClient request method
      federationClient.request = jest.fn().mockResolvedValueOnce({
        data: {
          workflow: {
            id: '123',
            name: 'Test Workflow',
            nodes: [{ id: 'node1', type: 'AI_AGENT' }],
            processedContent: {
              id: 'content1',
              summary: 'Test summary',
              extractedEntities: ['entity1', 'entity2']
            },
            relatedKnowledge: {
              concepts: ['concept1'],
              relationships: [{ from: 'entity1', to: 'concept1', type: 'RELATES_TO' }]
            }
          }
        }
      });

      const result = await federationClient.query(federatedQuery, { workflowId: '123' });

      expect(result.data.workflow).toBeDefined();
      expect(result.data.workflow.processedContent).toBeDefined();
      expect(result.data.workflow.relatedKnowledge).toBeDefined();
    });

    it('should handle partial service failures gracefully', async () => {
      const federatedQuery = `
        query GetWorkflowWithOptionalData($workflowId: ID!) {
          workflow(id: $workflowId) {
            id
            name
            # This might fail if content service is down
            processedContent {
              summary
            }
          }
        }
      `;

      // Mock a response with partial failures
      federationClient.request = jest.fn().mockRejectedValueOnce({
        data: {
          workflow: {
            id: '123',
            name: 'Test Workflow',
            processedContent: null
          }
        },
        errors: [{
          message: 'Content processing service unavailable',
          path: ['workflow', 'processedContent'],
          extensions: {
            code: 'SERVICE_UNAVAILABLE',
            service: 'content-processing'
          }
        }]
      });

      const result = await federationClient.query(federatedQuery, { workflowId: '123' });

      expect(result.data.workflow.id).toBe('123');
      expect(result.data.workflow.processedContent).toBeNull();
      expect(result.errors).toHaveLength(1);
      expect(result.errors![0].extensions?.service).toBe('content-processing');
    });

    it('should support entity resolution across services', async () => {
      const entityQuery = `
        query ResolveWorkflowEntities($representations: [_Any!]!) {
          _entities(representations: $representations) {
            ... on Workflow {
              id
              name
              status
            }
            ... on ProcessedContent {
              id
              workflowId
              summary
            }
          }
        }
      `;

      // Mock entity resolution
      federationClient.request = jest.fn().mockResolvedValueOnce({
        data: {
          _entities: [
            { __typename: 'Workflow', id: '123', name: 'Test', status: 'ACTIVE' },
            { __typename: 'ProcessedContent', id: 'c1', workflowId: '123', summary: 'Summary' }
          ]
        }
      });

      const representations = [
        { __typename: 'Workflow', id: '123' },
        { __typename: 'ProcessedContent', id: 'c1' }
      ];

      const result = await federationClient.query(entityQuery, { representations });

      expect(result.data._entities).toHaveLength(2);
      expect(result.data._entities[0].__typename).toBe('Workflow');
      expect(result.data._entities[1].__typename).toBe('ProcessedContent');
    });
  });

  describe('Service Health Monitoring', () => {
    it('should check gateway health status', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          status: 'healthy',
          services: {
            'workflow-api': { status: 'up', latency: 12 },
            'content-processing': { status: 'up', latency: 15 },
            'knowledge-graph': { status: 'up', latency: 20 },
            'realtime-communication': { status: 'up', latency: 8 }
          }
        })
      });

      const health = await federationClient.checkHealth();

      expect(health.status).toBe('healthy');
      expect(Object.keys(health.services)).toHaveLength(4);
      expect(health.services['workflow-api'].status).toBe('up');
    });

    it('should report degraded status when services are down', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          status: 'degraded',
          services: {
            'workflow-api': { status: 'up', latency: 12 },
            'content-processing': { status: 'down', error: 'Connection refused' },
            'knowledge-graph': { status: 'up', latency: 20 },
            'realtime-communication': { status: 'up', latency: 8 }
          }
        })
      });

      const health = await federationClient.checkHealth();

      expect(health.status).toBe('degraded');
      expect(health.services['content-processing'].status).toBe('down');
      expect(health.services['content-processing'].error).toBe('Connection refused');
    });
  });

  describe('Subscription Support', () => {
    it('should handle federated subscriptions', async () => {
      const subscription = `
        subscription OnWorkflowUpdate($workflowId: ID!) {
          workflowUpdated(id: $workflowId) {
            id
            status
            # Real-time updates from realtime-communication service
            activeUsers {
              id
              presence
            }
          }
        }
      `;

      const mockWebSocket = {
        send: jest.fn(),
        close: jest.fn(),
        addEventListener: jest.fn((event: string, callback: Function) => {
          if (event === 'open') {
            // Simulate WebSocket open immediately
            setTimeout(() => callback(), 0);
          }
        }),
        readyState: WebSocket.OPEN
      };

      (globalThis as any).WebSocket = jest.fn(() => mockWebSocket);

      const unsubscribe = await federationClient.subscribeFederated(
        subscription,
        { workflowId: '123' },
        {
          next: (data) => {
            expect(data.workflowUpdated).toBeDefined();
            expect(data.workflowUpdated.activeUsers).toBeDefined();
          },
          error: jest.fn(),
          complete: jest.fn()
        }
      );

      // Wait a tick for the WebSocket to be opened
      await new Promise(resolve => setTimeout(resolve, 10));
      
      expect(mockWebSocket.send).toHaveBeenCalled();
      expect(typeof unsubscribe).toBe('function');
    });
  });
});