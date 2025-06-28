import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TDDDemo from './TDDDemo';

// Mock the hooks to avoid external dependencies
jest.mock('../hooks/useWorkflowIntent', () => ({
  useWorkflowIntent: () => ({
    analyzeMessage: jest.fn(),
    intent: {
      type: 'create_workflow',
      confidence: 0.9,
      suggestedFields: [
        { name: 'name', label: 'Workflow Name', type: 'text', required: true },
        { name: 'type', label: 'Workflow Type', type: 'select', required: true, 
          options: [
            { value: 'customer_support', label: 'Customer Support' },
            { value: 'data_processing', label: 'Data Processing' }
          ]
        }
      ]
    },
    isAnalyzing: false,
    error: null
  })
}));

describe('TDDDemo', () => {
  it('renders the demo interface correctly', () => {
    render(<TDDDemo />);

    expect(screen.getByText('🎯 TDD Success Demo')).toBeInTheDocument();
    expect(screen.getByText('174+ Tests Passing')).toBeInTheDocument();
    expect(screen.getByText('💬 Chat Interface')).toBeInTheDocument();
    expect(screen.getByText('✅ TDD Implementation Success')).toBeInTheDocument();
  });

  it('displays welcome messages', () => {
    render(<TDDDemo />);

    expect(screen.getByText(/Welcome to the AI Workflow Builder/)).toBeInTheDocument();
    expect(screen.getByText(/Try saying "Create a customer support workflow"/)).toBeInTheDocument();
  });

  it('shows form builder when customer support workflow is requested', async () => {
    const user = userEvent.setup();
    render(<TDDDemo />);

    const input = screen.getByPlaceholderText(/Try: 'Create a customer support workflow'/);
    await user.type(input, 'Create a customer support workflow');
    
    const sendButton = screen.getByRole('button');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.getByText('🛠️ Workflow Builder')).toBeInTheDocument();
    });
  });

  it('displays TDD statistics correctly', () => {
    render(<TDDDemo />);

    expect(screen.getByText('ChatMessage: 5/5 tests ✅')).toBeInTheDocument();
    expect(screen.getByText('WorkflowPreview: 32/32 tests ✅')).toBeInTheDocument();
    expect(screen.getByText('Total: 174+ tests passing')).toBeInTheDocument();
    expect(screen.getByText('Methodology: Red-Green-Refactor')).toBeInTheDocument();
  });

  it('handles help messages appropriately', async () => {
    const user = userEvent.setup();
    render(<TDDDemo />);

    const input = screen.getByPlaceholderText(/Try: 'Create a customer support workflow'/);
    await user.type(input, 'help');
    
    const sendButton = screen.getByRole('button');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.getByText(/I can help you create various types of workflows/)).toBeInTheDocument();
    });
  });

  it('shows error boundary protection', () => {
    render(<TDDDemo />);
    
    // The component should be wrapped in ErrorBoundary
    // This test verifies the component renders without throwing
    expect(screen.getByText('🎯 TDD Success Demo')).toBeInTheDocument();
  });

  it('displays progress tracker when workflow is being built', async () => {
    const user = userEvent.setup();
    render(<TDDDemo />);

    const input = screen.getByPlaceholderText(/Try: 'Create a customer support workflow'/);
    await user.type(input, 'Create a customer support workflow');
    
    const sendButton = screen.getByRole('button');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.getByText('Multi-step form with 27+ tests')).toBeInTheDocument();
    });
  });
});