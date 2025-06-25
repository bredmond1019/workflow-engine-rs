import { Page, Route } from '@playwright/test';
import * as mocks from '../mocks/api-mocks';

/**
 * Test helper utilities for e2e tests
 */

export class TestHelpers {
  constructor(private page: Page) {}

  /**
   * Mock API endpoints with test data
   */
  async mockAPIEndpoints() {
    // Mock authentication endpoints
    await this.page.route('**/auth/token', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockAuthToken)
      });
    });

    await this.page.route('**/auth/verify', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          valid: true,
          claims: mocks.mockUser
        })
      });
    });

    // Mock workflow endpoints
    await this.page.route('**/api/v1/workflows/available', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockAvailableWorkflows)
      });
    });

    await this.page.route('**/api/v1/workflows/trigger', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockWorkflowTriggerResponse)
      });
    });

    await this.page.route('**/api/v1/workflows/instances', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ instances: [] })
      });
    });
  }

  /**
   * Mock workflow status progression
   * Simulates real-time updates from backend
   */
  async mockWorkflowStatusProgression(instanceId: string) {
    let statusIndex = 0;
    
    await this.page.route(`**/api/v1/workflows/status/${instanceId}`, async (route) => {
      const response = {
        ...mocks.mockWorkflowStatusProgression[Math.min(statusIndex, mocks.mockWorkflowStatusProgression.length - 1)],
        instance_id: instanceId
      };
      
      statusIndex++;
      
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(response)
      });
    });
  }

  /**
   * Mock LLM API calls
   * NOTE: In production, these would be real calls requiring API keys
   */
  async mockLLMAPICalls() {
    // Mock OpenAI API
    await this.page.route('**/api.openai.com/**', async (route) => {
      console.log('🤖 MOCK: OpenAI API call intercepted - would require OPENAI_API_KEY in production');
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockOpenAIResponse)
      });
    });

    // Mock Anthropic API
    await this.page.route('**/api.anthropic.com/**', async (route) => {
      console.log('🤖 MOCK: Anthropic API call intercepted - would require ANTHROPIC_API_KEY in production');
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockAnthropicResponse)
      });
    });

    // Mock AWS Bedrock
    await this.page.route('**/bedrock-runtime.*.amazonaws.com/**', async (route) => {
      console.log('🤖 MOCK: AWS Bedrock call intercepted - would require AWS credentials in production');
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockBedrockResponse)
      });
    });
  }

  /**
   * Mock MCP Server calls
   * NOTE: In production, these would be real calls to MCP servers
   */
  async mockMCPServerCalls() {
    // Mock HelpScout MCP Server
    await this.page.route('**/localhost:8001/**', async (route) => {
      console.log('🔌 MOCK: HelpScout MCP server call - would require running MCP server in production');
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockHelpScoutMCPResponse)
      });
    });

    // Mock Notion MCP Server
    await this.page.route('**/localhost:8002/**', async (route) => {
      console.log('🔌 MOCK: Notion MCP server call - would require running MCP server in production');
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockNotionMCPResponse)
      });
    });

    // Mock Slack MCP Server
    await this.page.route('**/localhost:8003/**', async (route) => {
      console.log('🔌 MOCK: Slack MCP server call - would require running MCP server in production');
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mocks.mockSlackMCPResponse)
      });
    });
  }

  /**
   * Login helper
   */
  async login(username: string = 'test_user', role: string = 'admin') {
    await this.page.goto('/login');
    await this.page.fill('input[placeholder="Enter username"]', username);
    await this.page.selectOption('select', role);
    await this.page.click('button[type="submit"]');
    await this.page.waitForURL('/dashboard');
  }

  /**
   * Wait for workflow to complete
   */
  async waitForWorkflowCompletion(timeout: number = 30000) {
    await this.page.waitForSelector('text=Completed', { timeout });
  }

  /**
   * Capture workflow execution metrics
   */
  async captureWorkflowMetrics() {
    const metrics = await this.page.evaluate(() => {
      // In production, these would be real metrics from the backend
      return {
        llmApiCalls: {
          openai: 2,
          anthropic: 1,
          bedrock: 0
        },
        mcpServerCalls: {
          helpscout: 1,
          notion: 1,
          slack: 1
        },
        totalTokensUsed: 425,
        estimatedCost: 0.0425, // $0.0425 based on token usage
        executionTime: 45000 // 45 seconds
      };
    });
    
    return metrics;
  }
}