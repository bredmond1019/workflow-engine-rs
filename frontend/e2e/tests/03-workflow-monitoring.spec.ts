import { test, expect } from '@playwright/test';
import { TestHelpers } from '../fixtures/test-helpers';

test.describe('Workflow Monitoring and Status', () => {
  let helpers: TestHelpers;

  test.beforeEach(async ({ page }) => {
    helpers = new TestHelpers(page);
    await helpers.mockAPIEndpoints();
    await helpers.mockLLMAPICalls();
    await helpers.mockMCPServerCalls();
    await helpers.login();
  });

  test('should display workflow detail page', async ({ page }) => {
    const instanceId = 'test-workflow-123';
    await helpers.mockWorkflowStatusProgression(instanceId);
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Should show workflow header
    await expect(page.locator('h1:has-text("Customer Care Workflow")')).toBeVisible();
    await expect(page.locator(`text=${instanceId}`)).toBeVisible();
    
    // Should show overview cards
    await expect(page.locator('text=Status')).toBeVisible();
    await expect(page.locator('text=Progress')).toBeVisible();
    await expect(page.locator('text=Duration')).toBeVisible();
    await expect(page.locator('text=Created')).toBeVisible();
  });

  test('should show real-time workflow progress', async ({ page }) => {
    const instanceId = 'test-workflow-123';
    await helpers.mockWorkflowStatusProgression(instanceId);
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Initial state - should be running
    await expect(page.locator('text=Running').first()).toBeVisible();
    
    // Wait for polling to update status
    await page.waitForTimeout(3000);
    
    // Should show step progress
    await expect(page.locator('text=Fetch Ticket Details')).toBeVisible();
    await expect(page.locator('text=Analyze With Ai')).toBeVisible();
    
    // Progress should update
    await expect(page.locator('text=40%')).toBeVisible();
  });

  test('should display step outputs', async ({ page }) => {
    const instanceId = 'test-workflow-123';
    await helpers.mockWorkflowStatusProgression(instanceId);
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Wait for steps to complete
    await page.waitForTimeout(5000);
    
    // Should show step outputs
    await expect(page.locator('text=Output:')).toBeVisible();
    
    // NOTE: These outputs come from real API calls in production
    console.log('\n🔍 STEP OUTPUTS THAT WOULD COME FROM REAL APIS:');
    console.log('================================================');
    console.log('1. HelpScout MCP Server: Ticket details, customer info');
    console.log('2. OpenAI/Anthropic API: AI analysis and sentiment');
    console.log('3. Notion MCP Server: Knowledge base articles');
    console.log('4. Anthropic API: Generated customer response');
    console.log('5. Slack MCP Server: Team notification confirmation');
    console.log('================================================\n');
  });

  test('should handle workflow completion', async ({ page }) => {
    const instanceId = 'test-workflow-123';
    await helpers.mockWorkflowStatusProgression(instanceId);
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Wait for workflow to complete
    await helpers.waitForWorkflowCompletion();
    
    // Should show completed status
    await expect(page.locator('text=Completed').first()).toBeVisible();
    
    // Should show final output
    await expect(page.locator('text=Final Output:')).toBeVisible();
    
    // Check final outputs
    const finalOutput = page.locator('pre').filter({ hasText: 'ticket_resolved' });
    await expect(finalOutput).toBeVisible();
  });

  test('should handle workflow errors', async ({ page }) => {
    const instanceId = 'test-error-workflow';
    
    // Mock error status
    await page.route(`**/api/v1/workflows/status/${instanceId}`, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          instance_id: instanceId,
          workflow_name: 'customer_care_workflow',
          status: 'Failed',
          created_at: new Date().toISOString(),
          error: {
            message: 'LLM API key invalid or expired',
            code: 'API_KEY_ERROR',
            step_id: 'analyze_with_ai',
            details: {
              provider: 'openai',
              error_type: 'authentication_error'
            }
          },
          progress: {
            total_steps: 5,
            completed_steps: 1,
            failed_steps: 1,
            running_steps: 0,
            percentage: 20
          }
        })
      });
    });
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Should show error status
    await expect(page.locator('text=Failed').first()).toBeVisible();
    
    // Should show error details
    await expect(page.locator('text=Workflow Error')).toBeVisible();
    await expect(page.locator('text=LLM API key invalid or expired')).toBeVisible();
    
    console.log('\n⚠️  ERROR SCENARIO DETECTED:');
    console.log('============================');
    console.log('Error: LLM API key invalid or expired');
    console.log('Step: analyze_with_ai');
    console.log('Provider: OpenAI');
    console.log('\nREQUIRED ACTION: Set OPENAI_API_KEY environment variable');
    console.log('============================\n');
  });

  test('should refresh workflow status', async ({ page }) => {
    const instanceId = 'test-workflow-123';
    await helpers.mockWorkflowStatusProgression(instanceId);
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Click refresh button
    await page.click('button:has-text("Refresh")');
    
    // Should show loading state
    await expect(page.locator('.ant-spin')).toBeVisible();
    
    // Status should update
    await page.waitForTimeout(1000);
    await expect(page.locator('text=40%')).toBeVisible();
  });

  test('should show workflow timing information', async ({ page }) => {
    const instanceId = 'test-workflow-123';
    await helpers.mockWorkflowStatusProgression(instanceId);
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Check timing information
    await expect(page.locator('text=Created At')).toBeVisible();
    await expect(page.locator('text=Started At')).toBeVisible();
    
    // Wait for completion
    await helpers.waitForWorkflowCompletion();
    
    // Should show completed time
    await expect(page.locator('text=Completed At')).toBeVisible();
    await expect(page.locator('text=Duration')).toBeVisible();
  });

  test('should navigate back to workflow list', async ({ page }) => {
    const instanceId = 'test-workflow-123';
    await helpers.mockWorkflowStatusProgression(instanceId);
    
    await page.goto(`/workflows/${instanceId}`);
    
    // Click back button
    await page.click('button:has-text("Back to Workflows")');
    
    // Should navigate to workflows page
    await expect(page).toHaveURL('/workflows');
  });

  test('should update dashboard metrics', async ({ page }) => {
    // Mock instances with various statuses
    await page.route('**/api/v1/workflows/instances', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          instances: [
            {
              instance_id: 'workflow-1',
              workflow_name: 'customer_care_workflow',
              status: 'Completed',
              created_at: new Date().toISOString()
            },
            {
              instance_id: 'workflow-2',
              workflow_name: 'research_to_documentation',
              status: 'Running',
              created_at: new Date().toISOString()
            },
            {
              instance_id: 'workflow-3',
              workflow_name: 'knowledge_base_workflow',
              status: 'Failed',
              created_at: new Date().toISOString()
            }
          ]
        })
      });
    });
    
    await page.goto('/dashboard');
    
    // Should show updated metrics
    await expect(page.locator('text=Total Workflows')).toBeVisible();
    await expect(page.locator('.ant-statistic-content:has-text("3")')).toBeVisible();
    
    // Check individual counts
    await expect(page.locator('.ant-statistic-content:has-text("1")').first()).toBeVisible(); // Completed
    await expect(page.locator('.ant-statistic-content:has-text("1")').nth(1)).toBeVisible(); // Running
    await expect(page.locator('.ant-statistic-content:has-text("1")').nth(2)).toBeVisible(); // Failed
  });

  test('should filter workflows by status', async ({ page }) => {
    // Mock instances
    await page.route('**/api/v1/workflows/instances', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          instances: [
            {
              instance_id: 'workflow-1',
              workflow_name: 'customer_care_workflow',
              status: 'Completed',
              created_at: new Date().toISOString(),
              progress: { percentage: 100 }
            },
            {
              instance_id: 'workflow-2',
              workflow_name: 'research_to_documentation',
              status: 'Running',
              created_at: new Date().toISOString(),
              progress: { percentage: 60 }
            }
          ]
        })
      });
    });
    
    await page.goto('/workflows');
    
    // Filter by running
    await page.selectOption('select', 'Running');
    
    // Should only show running workflows
    await expect(page.locator('text=research_to_documentation')).toBeVisible();
    await expect(page.locator('text=customer_care_workflow')).not.toBeVisible();
    
    // Filter by completed
    await page.selectOption('select', 'Completed');
    
    // Should only show completed workflows
    await expect(page.locator('text=customer_care_workflow')).toBeVisible();
    await expect(page.locator('text=research_to_documentation')).not.toBeVisible();
  });
});