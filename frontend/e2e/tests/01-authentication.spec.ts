import { test, expect } from '@playwright/test';
import { TestHelpers } from '../fixtures/test-helpers';

test.describe('Authentication Flow', () => {
  let helpers: TestHelpers;

  test.beforeEach(async ({ page }) => {
    helpers = new TestHelpers(page);
    await helpers.mockAPIEndpoints();
  });

  test('should display login page with all elements', async ({ page }) => {
    await page.goto('/login');
    
    // Check all UI elements are present
    await expect(page.locator('h1:has-text("AI Workflow Engine")')).toBeVisible();
    await expect(page.locator('input[placeholder="Enter username"]')).toBeVisible();
    await expect(page.locator('select')).toBeVisible();
    await expect(page.locator('button:has-text("Sign In")')).toBeVisible();
  });

  test('should successfully login with valid credentials', async ({ page }) => {
    await page.goto('/login');
    
    // Fill login form
    await page.fill('input[placeholder="Enter username"]', 'test_user');
    await page.selectOption('select', 'admin');
    
    // Submit form
    await page.click('button:has-text("Sign In")');
    
    // Should redirect to dashboard
    await expect(page).toHaveURL('/dashboard');
    
    // Should show user info in header
    await expect(page.locator('text=test_user')).toBeVisible();
  });

  test('should handle login errors gracefully', async ({ page }) => {
    // Mock error response
    await page.route('**/auth/token', async (route) => {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'invalid_credentials',
          message: 'Invalid username or password'
        })
      });
    });
    
    await page.goto('/login');
    await page.fill('input[placeholder="Enter username"]', 'invalid_user');
    await page.selectOption('select', 'admin');
    await page.click('button:has-text("Sign In")');
    
    // Should show error message
    await expect(page.locator('text=Authentication failed')).toBeVisible();
  });

  test('should persist authentication state', async ({ page }) => {
    await helpers.login();
    
    // Reload page
    await page.reload();
    
    // Should still be on dashboard
    await expect(page).toHaveURL('/dashboard');
  });

  test('should logout successfully', async ({ page }) => {
    await helpers.login();
    
    // Click user avatar
    await page.click('.ant-avatar');
    
    // Click logout
    await page.click('text=Logout');
    
    // Should redirect to login
    await expect(page).toHaveURL('/login');
  });

  test('should redirect to login when token expires', async ({ page }) => {
    await helpers.login();
    
    // Mock 401 response to simulate expired token
    await page.route('**/api/v1/workflows/instances', async (route) => {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'token_expired',
          message: 'Token has expired'
        })
      });
    });
    
    // Navigate to workflows page which will make API call
    await page.goto('/workflows');
    
    // Should redirect to login
    await expect(page).toHaveURL('/login');
  });

  test('should support different user roles', async ({ page }) => {
    const roles = ['admin', 'developer', 'analyst', 'viewer'];
    
    for (const role of roles) {
      await page.goto('/login');
      await page.fill('input[placeholder="Enter username"]', `test_${role}`);
      await page.selectOption('select', role);
      await page.click('button:has-text("Sign In")');
      
      // Should login successfully
      await expect(page).toHaveURL('/dashboard');
      
      // Should show role in header
      await expect(page.locator(`text=(${role})`)).toBeVisible();
      
      // Logout for next iteration
      await page.click('.ant-avatar');
      await page.click('text=Logout');
    }
  });
});