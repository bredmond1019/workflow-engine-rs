# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Development
```bash
# Start development server
npm run dev

# Build for production
npm run build

# Run linting
npm run lint

# Type checking
npm run type-check

# Preview production build
npm run preview

# Build with bundle analysis
npm run build:analyze
```

### Testing
```bash
# Run all E2E tests
npm run test:e2e

# Run E2E tests with UI (interactive)
npm run test:e2e:ui

# Run E2E tests in headed mode (see browser)
npm run test:e2e:headed

# Debug E2E tests
npm run test:e2e:debug

# Run specific test suite
npm run test:e2e -- --grep "Authentication"
```

### Important Notes
- **Vite Version**: Uses Vite 4.4.0 for Node.js 18 compatibility (do not upgrade to v7+)
- **PostCSS**: Uses `@tailwindcss/postcss` with array syntax configuration
- **Production Mode**: Connects to backend API at http://localhost:8080
- **Backend Required**: Authentication and workflow features require running backend services

## Architecture Overview

### Frontend Stack
- **React 19** with TypeScript for type safety
- **Vite 4.4.0** as build tool (pinned for Node.js compatibility)
- **Ant Design 5.26.2** for enterprise UI components
- **Tailwind CSS 4.1.10** for utility styling (with preflight disabled)
- **Zustand 5.0.5** for state management with persistence
- **Axios** for API client with interceptors
- **React Router 6** for navigation

### Project Structure
```
src/
├── api/           # API client configuration and endpoints
├── components/    # Shared layout and auth components
├── features/      # Feature-based page components
├── stores/        # Zustand state management
├── types/         # TypeScript type definitions
└── utils/         # Utility functions
```

### State Management Architecture

The application uses **Zustand** for state management with two main stores:

#### AuthStore (`src/stores/authStore.ts`)
- Handles JWT-based authentication with localStorage persistence
- Methods: `login()`, `logout()`, `verifyToken()`
- Persists authentication state across page reloads
- Automatic token validation on app startup

#### WorkflowStore (`src/stores/workflowStore.ts`)
- Manages workflow templates, instances, and execution
- Real-time polling for workflow status updates (2 second intervals)
- Methods: `triggerWorkflow()`, `fetchWorkflowStatus()`, `updateInstance()`, `startPolling()`, `stopPolling()`
- Tracks workflow progress with step-by-step updates

### API Integration Pattern

The API client (`src/api/config.ts`) implements:
- **Automatic token injection** via request interceptors
- **401 handling** with automatic logout and redirect
- **Environment-based configuration** via `VITE_API_URL`
- **Comprehensive endpoint mapping** for all backend services

### Backend Integration

The frontend is designed to work with the AI Workflow Engine backend:
- **Authentication**: JWT tokens from `/auth/token` endpoint
- **Workflow Management**: Real-time status from `/api/v1/workflows/*` endpoints
- **Available Workflows**: Dynamic list from backend configuration
- **Step Execution**: Detailed progress tracking with outputs

### TypeScript Configuration

Key patterns used:
- **Const objects instead of enums** for better compatibility with `erasableSyntaxOnly`
- **Type-only imports** for interfaces and types
- **Strict mode enabled** with comprehensive type checking
- **References-based project structure** with separate app/node configs

### Styling Architecture

#### Tailwind + Ant Design Integration
- **Tailwind preflight disabled** to prevent conflicts with Ant Design
- **PostCSS v4 configuration** using `@tailwindcss/postcss`
- **Array syntax** in `postcss.config.js` for proper plugin loading
- **Ant Design theming** via ConfigProvider with custom tokens

### Component Organization

#### Feature-Based Structure
- **`features/`** contains page-level components grouped by functionality
- **`components/`** contains shared layout and authentication components
- **Nested routing** with protected routes and layout integration

#### Key Components
- **MainLayout**: Sidebar navigation with Ant Design layout (Dashboard + Workflows)
- **ProtectedRoute**: Authentication guard for secured pages
- **DashboardPage**: Executive metrics with real-time statistics
- **WorkflowsPage**: Workflow list with trigger functionality
- **WorkflowDetailPage**: Step-by-step execution visualization with outputs

### Configuration Files

#### Critical Configurations
- **`vite.config.ts`**: Basic Vite setup with React plugin
- **`tailwind.config.js`**: Tailwind configuration with preflight disabled
- **`postcss.config.js`**: PostCSS setup for Tailwind CSS v4
- **`tsconfig.json`**: References-based TypeScript project structure

## Development Patterns

### State Updates
Always use Zustand's `set()` function for state mutations:
```typescript
set((state) => {
  const newInstances = new Map(state.instances);
  newInstances.set(instanceId, instance);
  return { instances: newInstances };
});
```

### API Error Handling
Implement graceful fallbacks for API failures:
```typescript
} catch (error: any) {
  console.warn('API not available, using demo mode');
  // Fallback implementation
}
```

### Type Definitions
Use const objects for enums to avoid TypeScript compilation issues:
```typescript
export const WorkflowStatus = {
  Created: 'Created',
  Running: 'Running',
  // ...
} as const;

export type WorkflowStatus = typeof WorkflowStatus[keyof typeof WorkflowStatus];
```

### Workflow Status Updates
Handle real-time workflow status updates:
```typescript
// Workflow status progression
const instance = await fetchWorkflowStatus(instanceId);
// Updates steps, progress, and completion status
```

## Troubleshooting

### Common Issues

#### Vite Version Compatibility
- **Error**: `crypto.hash is not a function`
- **Solution**: Ensure Vite is at version 4.4.0, not 7.x+

#### PostCSS/Tailwind Configuration
- **Error**: "trying to use tailwindcss directly as a PostCSS plugin"
- **Solution**: Use `@tailwindcss/postcss` with array syntax in config

#### TypeScript Strict Mode
- **Error**: Enum syntax not allowed with `erasableSyntaxOnly`
- **Solution**: Use const object pattern instead of enums

#### Authentication Errors
- **Issue**: "Authentication failed" message
- **Solution**: Ensure backend is running and JWT_SECRET is configured

### Environment Setup
```bash
# Required for Node.js 18 compatibility
npm install vite@^4.4.0

# Required for Tailwind CSS v4
npm install @tailwindcss/postcss
```

## Business Context

This frontend serves as an **executive demonstration platform** for the AI workflow orchestration backend. Key business features:

- **Real-time dashboards** showing workflow execution metrics
- **Interactive demos** with predefined business scenarios
- **Professional UI** suitable for C-level presentations
- **Standalone operation** without requiring backend infrastructure
- **Performance metrics** highlighting business value propositions

The application emphasizes **business outcomes** over technical details, making it ideal for executive stakeholder demonstrations and investment presentations.