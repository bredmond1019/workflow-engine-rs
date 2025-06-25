import { test, expect } from '@playwright/test';
import { TestHelpers } from '../fixtures/test-helpers';

test.describe('Workflow Trigger and Execution', () => {
  let helpers: TestHelpers;

  test.beforeEach(async ({ page }) => {
    helpers = new TestHelpers(page);
    await helpers.mockAPIEndpoints();
    await helpers.mockLLMAPICalls();
    await helpers.mockMCPServerCalls();
    await helpers.login();
  });

  test('should display available workflows', async ({ page }) => {
    await page.goto('/workflows');
    
    // Click trigger button
    await page.click('button:has-text("Trigger Workflow")');
    
    // Should show workflow selection
    await expect(page.locator('text=Select Workflow')).toBeVisible();
    
    // Click on select dropdown
    await page.click('select');
    
    // Should show all available workflows
    await expect(page.locator('text=Customer Care Workflow')).toBeVisible();
    await expect(page.locator('text=Research To Documentation')).toBeVisible();
    await expect(page.locator('text=Knowledge Base Workflow')).toBeVisible();
  });

  test('should validate JSON input', async ({ page }) => {
    await page.goto('/workflows');
    await page.click('button:has-text("Trigger Workflow")');
    
    // Select workflow
    await page.selectOption('select', 'customer_care_workflow');
    
    // Enter invalid JSON
    await page.fill('textarea', 'invalid json');
    
    // Try to submit
    await page.click('button:has-text("Trigger")');
    
    // Should show validation error
    await expect(page.locator('text=Invalid JSON format')).toBeVisible();
  });

  test('should successfully trigger a workflow', async ({ page }) => {
    await page.goto('/workflows');
    
    // Open trigger drawer
    await page.click('button:has-text("Trigger Workflow")');
    
    // Select workflow
    await page.selectOption('select', 'customer_care_workflow');
    
    // Enter valid input
    const workflowInput = {
      ticket_id: 'HS-12345',
      priority: 'high',
      customer_email: 'test@example.com'
    };
    
    await page.fill('textarea', JSON.stringify(workflowInput, null, 2));
    
    // Submit
    await page.click('button:has-text("Trigger")');
    
    // Should show success message
    await expect(page.locator('text=Workflow triggered successfully')).toBeVisible();
    
    // Should redirect to workflow detail page
    await expect(page).toHaveURL(/\/workflows\/test-workflow-123/);
  });

  test('should trigger workflow from dashboard', async ({ page }) => {
    await page.goto('/dashboard');
    
    // Click new workflow button
    await page.click('button:has-text("New Workflow")');
    
    // Should navigate to workflows page
    await expect(page).toHaveURL('/workflows');
  });

  test('should show workflow in list after triggering', async ({ page }) => {
    // Mock instances endpoint to return our triggered workflow
    await page.route('**/api/v1/workflows/instances', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          instances: [{
            instance_id: 'test-workflow-123',
            workflow_name: 'customer_care_workflow',
            status: 'Running',
            created_at: new Date().toISOString(),
            inputs: { ticket_id: 'HS-12345' },
            progress: {
              total_steps: 5,
              completed_steps: 2,
              failed_steps: 0,
              running_steps: 1,
              percentage: 40
            }
          }]
        })
      });
    });
    
    await page.goto('/workflows');
    
    // Should show workflow in list
    await expect(page.locator('text=Customer Care Workflow')).toBeVisible();
    await expect(page.locator('text=test-workflow-123')).toBeVisible();
    await expect(page.locator('text=Running')).toBeVisible();
    await expect(page.locator('text=40%')).toBeVisible();
  });

  test('should handle workflow trigger errors', async ({ page }) => {
    // Mock error response
    await page.route('**/api/v1/workflows/trigger', async (route) => {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'invalid_workflow',
          message: 'Workflow not found or invalid configuration'
        })
      });
    });
    
    await page.goto('/workflows');
    await page.click('button:has-text("Trigger Workflow")');
    await page.selectOption('select', 'customer_care_workflow');
    await page.fill('textarea', JSON.stringify({ test: 'data' }, null, 2));
    await page.click('button:has-text("Trigger")');
    
    // Should show error message
    await expect(page.locator('text=Failed to trigger workflow')).toBeVisible();
  });

  test('should pre-fill input data template', async ({ page }) => {
    await page.goto('/workflows');
    await page.click('button:has-text("Trigger Workflow")');
    
    // Should have default input template
    const textareaValue = await page.inputValue('textarea');
    const parsedValue = JSON.parse(textareaValue);
    
    expect(parsedValue).toHaveProperty('query');
    expect(parsedValue).toHaveProperty('priority');
  });

  test('should allow configuration overrides', async ({ page }) => {
    await page.goto('/workflows');
    await page.click('button:has-text("Trigger Workflow")');
    
    await page.selectOption('select', 'customer_care_workflow');
    
    // Fill configuration overrides
    const configOverrides = {
      timeout: 600,
      retries: 5,
      continue_on_error: true
    };
    
    await page.fill('textarea:last-of-type', JSON.stringify(configOverrides, null, 2));
    
    // Submit
    await page.click('button:has-text("Trigger")');
    
    // Should succeed
    await expect(page.locator('text=Workflow triggered successfully')).toBeVisible();
  });

  test('should capture workflow trigger metrics', async ({ page }) => {
    console.log('\n📊 WORKFLOW TRIGGER METRICS:');
    console.log('================================');
    
    await page.goto('/workflows');
    await page.click('button:has-text("Trigger Workflow")');
    await page.selectOption('select', 'customer_care_workflow');
    await page.fill('textarea', JSON.stringify({ ticket_id: 'HS-12345' }, null, 2));
    await page.click('button:has-text("Trigger")');
    
    // Capture metrics
    const metrics = await helpers.captureWorkflowMetrics();
    
    console.log('🔑 API Keys Required in Production:');
    console.log('  - OPENAI_API_KEY: For GPT-4 analysis');
    console.log('  - ANTHROPIC_API_KEY: For Claude response generation');
    console.log('  - HELPSCOUT_API_KEY: For ticket management');
    console.log('  - NOTION_API_KEY: For knowledge base access');
    console.log('  - SLACK_BOT_TOKEN: For team notifications');
    
    console.log('\n💰 Estimated Costs:');
    console.log(`  - Total tokens used: ${metrics.totalTokensUsed}`);
    console.log(`  - Estimated cost: $${metrics.estimatedCost.toFixed(4)}`);
    console.log(`  - Execution time: ${metrics.executionTime / 1000}s`);
    
    console.log('\n🔌 External Service Calls:');
    console.log(`  - LLM API calls: ${JSON.stringify(metrics.llmApiCalls)}`);
    console.log(`  - MCP server calls: ${JSON.stringify(metrics.mcpServerCalls)}`);
    console.log('================================\n');
  });
});