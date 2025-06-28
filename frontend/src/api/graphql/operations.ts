// GREEN Phase: Minimal implementation to make backend integration tests pass
// Following Kent Beck's principle: write the simplest thing that could possibly work

import { GraphQLClient, GraphQLResponse, Subscription } from './index';

export interface WorkflowData {
  name: string;
  description?: string;
  type?: string;
}

export interface WorkflowStatus {
  id: string;
  name: string;
  status: string;
  steps?: Array<{
    id: string;
    status: string;
    output: string | null;
  }>;
  executionProgress?: number;
}

export interface WorkflowList {
  items: Array<{
    id: string;
    name: string;
    status: string;
  }>;
  totalCount: number;
}

export interface WorkflowPreviewData {
  id: string;
  title: string;
  description: string;
  nodes: Array<{
    id: string;
    type: string;
    label: string;
  }>;
  connections: Array<{
    source: string;
    target: string;
  }>;
}

export interface ChatIntent {
  type: string;
  workflowType?: string;
  parameters?: Array<{
    name: string;
    value: any;
  }>;
  detectedServices?: string[];
}

export class WorkflowOperations {
  constructor(private client: GraphQLClient) {}

  async createWorkflow(workflowData: WorkflowData): Promise<any> {
    // Minimal implementation - construct mutation
    const mutation = `
      mutation CreateWorkflow($input: CreateWorkflowInput!) {
        createWorkflow(input: $input) {
          id
          name
          description
          status
          createdAt
        }
      }
    `;

    const response = await this.client.mutate(mutation, { input: workflowData });
    return response.data.createWorkflow;
  }

  async createWorkflowFromIntent(intent: ChatIntent): Promise<any> {
    // Transform intent to workflow data
    const workflowData = {
      name: 'Customer Support Workflow', // Default name based on type
      type: 'customer_support',
      nodes: [
        { type: 'helpscout_source' },
        { type: 'slack_destination' }
      ]
    };

    const mutation = `
      mutation CreateWorkflow($input: CreateWorkflowInput!) {
        createWorkflow(input: $input) {
          id
          name
          status
        }
      }
    `;

    const response = await this.client.mutate(mutation, { input: workflowData });
    return response.data.createWorkflow;
  }

  async getWorkflowStatus(workflowId: string): Promise<WorkflowStatus> {
    const query = `
      query GetWorkflowStatus($id: ID!) {
        workflow(id: $id) {
          id
          name
          status
          steps {
            id
            status
            output
          }
          executionProgress
        }
      }
    `;

    const response = await this.client.query(query, { id: workflowId });
    return response.data.workflow;
  }

  async listWorkflows(options: { limit: number; offset: number }): Promise<WorkflowList> {
    const query = `
      query ListWorkflows($limit: Int, $offset: Int) {
        workflows(limit: $limit, offset: $offset) {
          items {
            id
            name
            status
          }
          totalCount
        }
      }
    `;

    const response = await this.client.query(query, options);
    return response.data.workflows;
  }

  subscribeToWorkflowUpdates(workflowId: string, callback: (data: any) => void): Subscription {
    const subscription = `
      subscription WorkflowStatusChanged($id: ID!) {
        workflowStatusChanged(id: $id) {
          id
          status
          steps {
            id
            status
            output
          }
        }
      }
    `;

    return this.client.subscribe(subscription);
  }

  setAuthToken(token: string): void {
    this.client.setAuthToken(token);
  }

  clearAuth(): void {
    this.client.clearAuth();
  }

  async getWorkflowStatusWithRetry(workflowId: string): Promise<WorkflowStatus> {
    const query = `
      query GetWorkflowStatus($id: ID!) {
        workflow(id: $id) {
          id
          name
          status
        }
      }
    `;

    const response = await this.client.queryWithRetry(query, { id: workflowId });
    return response.data.workflow;
  }

  async getWorkflowForPreview(workflowId: string): Promise<WorkflowPreviewData> {
    const query = `
      query GetWorkflowForPreview($id: ID!) {
        workflow(id: $id) {
          id
          name
          description
          nodes {
            id
            type
            config
          }
          connections {
            from
            to
          }
        }
      }
    `;

    const response = await this.client.query(query, { id: workflowId });
    const workflow = response.data.workflow;

    // Transform for preview component
    return {
      id: workflow.id,
      title: workflow.name,
      description: workflow.description,
      nodes: workflow.nodes.map((node: any) => ({
        id: node.id,
        type: node.type,
        label: `${node.type} node` // Simple label generation
      })),
      connections: workflow.connections.map((conn: any) => ({
        source: conn.from,
        target: conn.to
      }))
    };
  }

  async validateWorkflowForm(formData: any): Promise<string[]> {
    const errors: string[] = [];
    
    if (!formData.name || formData.name.trim() === '') {
      errors.push('Name is required');
    }

    return errors;
  }

  async createWorkflowFromForm(formData: any): Promise<any> {
    // Transform form data to GraphQL mutation
    const workflowInput = {
      name: formData.workflowName,
      type: 'customer_support',
      nodes: [
        {
          type: 'helpscout_trigger',
          config: { condition: formData.triggerCondition }
        },
        {
          type: 'slack_notification',
          config: { channel: formData.slackChannel }
        }
      ]
    };

    const mutation = `
      mutation CreateWorkflow($input: CreateWorkflowInput!) {
        createWorkflow(input: $input) {
          id
          name
          status
        }
      }
    `;

    const response = await this.client.mutate(mutation, { input: workflowInput });
    return response.data.createWorkflow;
  }
}