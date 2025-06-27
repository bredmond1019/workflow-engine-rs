import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { WorkflowPreview } from './WorkflowPreview';
import { ChatMessage } from '../../types/chat';
import { WorkflowIntent } from '../../types/workflow';
import { IntentType, WorkflowType } from '../../../../services/ai/WorkflowIntentAnalyzer';

// Mock ResizeObserver
beforeAll(() => {
  (window as any).ResizeObserver = jest.fn().mockImplementation(() => ({
    observe: jest.fn(),
    unobserve: jest.fn(),
    disconnect: jest.fn(),
  }));
});

describe('WorkflowPreview', () => {
  const mockMessages: ChatMessage[] = [
    {
      id: '1',
      content: 'I want to create a customer support workflow',
      role: 'user',
      timestamp: new Date('2023-01-01T10:00:00Z'),
    },
    {
      id: '2',
      content: 'I\'ll help you create a customer support workflow. Let me set that up for you.',
      role: 'assistant',
      timestamp: new Date('2023-01-01T10:00:10Z'),
    },
  ];

  const mockIntent: WorkflowIntent = {
    type: IntentType.CREATE_WORKFLOW,
    confidence: 0.9,
    suggestedFields: [
      {
        name: 'workflowName',
        type: 'text',
        label: 'Workflow Name',
        required: true,
      },
      {
        name: 'triggerType',
        type: 'select',
        label: 'Trigger Type',
        required: true,
        options: [
          { value: 'webhook', label: 'Webhook' },
          { value: 'schedule', label: 'Schedule' },
          { value: 'manual', label: 'Manual' },
        ],
      },
    ],
    extractedEntities: {
      workflowType: WorkflowType.CUSTOMER_SUPPORT,
      services: ['helpscout', 'slack'],
    },
    parameters: {
      source: 'helpscout',
      destination: 'slack',
      condition: 'priority=high',
    },
  };

  const mockOnNodeClick = jest.fn();
  const mockOnConnectionClick = jest.fn();
  const mockOnNodeHover = jest.fn();

  const defaultProps = {
    messages: mockMessages,
    intent: mockIntent,
    currentStep: 'trigger',
    onNodeClick: mockOnNodeClick,
    onConnectionClick: mockOnConnectionClick,
    onNodeHover: mockOnNodeHover,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('6a. Renders workflow nodes from chat data', () => {
    it('should render trigger node based on extracted parameters', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      expect(triggerNode).toBeInTheDocument();
      expect(triggerNode).toHaveTextContent('Webhook Trigger');
      expect(triggerNode).toHaveClass('nodeTypeTrigger');
    });

    it('should render processing nodes for detected services', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const helpscoutNode = screen.getByTestId('workflow-node-helpscout');
      expect(helpscoutNode).toBeInTheDocument();
      expect(helpscoutNode).toHaveTextContent('HelpScout');
      expect(helpscoutNode).toHaveClass('nodeTypeSource');
    });

    it('should render condition node when conditions are present', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const conditionNode = screen.getByTestId('workflow-node-condition');
      expect(conditionNode).toBeInTheDocument();
      expect(conditionNode).toHaveTextContent('Priority = High');
      expect(conditionNode).toHaveClass('nodeTypeCondition');
    });

    it('should render output node for destination service', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const slackNode = screen.getByTestId('workflow-node-slack');
      expect(slackNode).toBeInTheDocument();
      expect(slackNode).toHaveTextContent('Send to Slack');
      expect(slackNode).toHaveClass('nodeTypeAction');
    });

    it('should render AI processing node when AI is detected', () => {
      const intentWithAI = {
        ...mockIntent,
        extractedEntities: {
          ...mockIntent.extractedEntities,
          hasAI: true,
          aiProvider: 'openai',
        },
      };
      
      render(<WorkflowPreview {...defaultProps} intent={intentWithAI} />);
      
      const aiNode = screen.getByTestId('workflow-node-ai');
      expect(aiNode).toBeInTheDocument();
      expect(aiNode).toHaveTextContent('AI Processing');
      expect(aiNode).toHaveClass('nodeTypeAI');
    });

    it('should handle empty workflow gracefully', () => {
      const emptyIntent: WorkflowIntent = {
        type: IntentType.UNKNOWN,
        confidence: 0,
        extractedEntities: {},
      };
      
      render(<WorkflowPreview {...defaultProps} intent={emptyIntent} />);
      
      const emptyState = screen.getByTestId('workflow-empty-state');
      expect(emptyState).toBeInTheDocument();
      expect(emptyState).toHaveTextContent('Start building your workflow');
    });
  });

  describe('6b. Updates preview in real-time', () => {
    it('should update nodes when intent changes', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} />);
      
      expect(screen.getByTestId('workflow-node-helpscout')).toBeInTheDocument();
      
      const updatedIntent = {
        ...mockIntent,
        extractedEntities: {
          ...mockIntent.extractedEntities,
          services: ['notion', 'slack'],
        },
      };
      
      rerender(<WorkflowPreview {...defaultProps} intent={updatedIntent} />);
      
      expect(screen.queryByTestId('workflow-node-helpscout')).not.toBeInTheDocument();
      expect(screen.getByTestId('workflow-node-notion')).toBeInTheDocument();
    });

    it('should add new nodes as conversation progresses', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} />);
      
      const nodeCount = screen.getAllByTestId(/^workflow-node-/).length;
      
      const updatedIntent = {
        ...mockIntent,
        extractedEntities: {
          ...mockIntent.extractedEntities,
          services: ['helpscout', 'slack', 'email'],
          hasTransformation: true,
        },
      };
      
      rerender(<WorkflowPreview {...defaultProps} intent={updatedIntent} />);
      
      const newNodeCount = screen.getAllByTestId(/^workflow-node-/).length;
      expect(newNodeCount).toBeGreaterThan(nodeCount);
      expect(screen.getByTestId('workflow-node-transformation')).toBeInTheDocument();
    });

    it('should update node properties based on form values', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} />);
      
      const updatedIntent = {
        ...mockIntent,
        parameters: {
          ...mockIntent.parameters,
          schedule: 'daily at 9am',
        },
      };
      
      rerender(<WorkflowPreview {...defaultProps} intent={updatedIntent} />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      expect(triggerNode).toHaveTextContent('Schedule: Daily at 9am');
    });

    it('should reflect multi-step progress in node states', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} currentStep="condition" />);
      
      const conditionNode = screen.getByTestId('workflow-node-condition');
      expect(conditionNode).toHaveClass('nodeActive');
      
      rerender(<WorkflowPreview {...defaultProps} currentStep="action" />);
      
      expect(conditionNode).not.toHaveClass('nodeActive');
      expect(conditionNode).toHaveClass('nodeCompleted');
      
      const actionNode = screen.getByTestId('workflow-node-slack');
      expect(actionNode).toHaveClass('nodeActive');
    });
  });

  describe('6c. Shows connections between nodes', () => {
    it('should render connection lines between sequential nodes', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const connections = screen.getAllByTestId(/^workflow-connection-/);
      expect(connections.length).toBeGreaterThan(0);
      
      // Check specific connection
      const triggerToSource = screen.getByTestId('workflow-connection-trigger-helpscout');
      expect(triggerToSource).toBeInTheDocument();
      expect(triggerToSource).toHaveClass('connection');
    });

    it('should show animated flow on active connections', () => {
      render(<WorkflowPreview {...defaultProps} currentStep="source" />);
      
      const activeConnection = screen.getByTestId('workflow-connection-trigger-helpscout');
      expect(activeConnection).toHaveClass('connectionActive');
    });

    it('should handle branching connections for conditions', () => {
      const intentWithMultipleConditions = {
        ...mockIntent,
        parameters: {
          ...mockIntent.parameters,
          conditions: [
            { field: 'priority', operator: 'equals', value: 'high' },
            { field: 'category', operator: 'equals', value: 'billing' },
          ],
        },
      };
      
      render(<WorkflowPreview {...defaultProps} intent={intentWithMultipleConditions} />);
      
      const conditionConnections = screen.getAllByTestId(/workflow-connection-condition-/);
      expect(conditionConnections.length).toBeGreaterThanOrEqual(2);
    });

    it('should update connection states based on workflow progress', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} currentStep="trigger" />);
      
      const connection = screen.getByTestId('workflow-connection-trigger-helpscout');
      expect(connection).not.toHaveClass('connectionCompleted');
      
      rerender(<WorkflowPreview {...defaultProps} currentStep="action" />);
      
      expect(connection).toHaveClass('connectionCompleted');
    });

    it('should show connection labels for data flow', () => {
      render(<WorkflowPreview {...defaultProps} showConnectionLabels />);
      
      const connectionLabel = screen.getByTestId('connection-label-trigger-helpscout');
      expect(connectionLabel).toBeInTheDocument();
      expect(connectionLabel).toHaveTextContent('Ticket Data');
    });
  });

  describe('6d. Highlights current configuration step', () => {
    it('should highlight the current node being configured', () => {
      render(<WorkflowPreview {...defaultProps} currentStep="trigger" />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      expect(triggerNode).toHaveClass('nodeActive');
      expect(triggerNode).toHaveClass('nodeHighlight');
      
      const otherNodes = screen.getAllByTestId(/^workflow-node-(?!trigger)/);
      otherNodes.forEach(node => {
        expect(node).not.toHaveClass('nodeActive');
      });
    });

    it('should show pulse animation on active node', () => {
      render(<WorkflowPreview {...defaultProps} currentStep="condition" />);
      
      const conditionNode = screen.getByTestId('workflow-node-condition');
      expect(conditionNode).toHaveClass('nodePulse');
    });

    it('should dim inactive nodes when highlighting current', () => {
      render(<WorkflowPreview {...defaultProps} currentStep="action" highlightMode="focus" />);
      
      const activeNode = screen.getByTestId('workflow-node-slack');
      expect(activeNode).not.toHaveClass('nodeDimmed');
      
      const inactiveNodes = screen.getAllByTestId(/^workflow-node-(?!slack)/);
      inactiveNodes.forEach(node => {
        expect(node).toHaveClass('nodeDimmed');
      });
    });

    it('should show configuration progress indicator on nodes', () => {
      const intentWithProgress = {
        ...mockIntent,
        nodeProgress: {
          trigger: 100,
          helpscout: 100,
          condition: 50,
          slack: 0,
        },
      };
      
      render(<WorkflowPreview {...defaultProps} intent={intentWithProgress} />);
      
      const conditionProgress = screen.getByTestId('node-progress-condition');
      expect(conditionProgress).toHaveAttribute('data-progress', '50');
      expect(conditionProgress).toHaveStyle({ width: '50%' });
    });

    it('should update highlight when step changes', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} currentStep="trigger" />);
      
      expect(screen.getByTestId('workflow-node-trigger')).toHaveClass('nodeActive');
      
      rerender(<WorkflowPreview {...defaultProps} currentStep="source" />);
      
      expect(screen.getByTestId('workflow-node-trigger')).not.toHaveClass('nodeActive');
      expect(screen.getByTestId('workflow-node-helpscout')).toHaveClass('nodeActive');
    });
  });

  describe('6e. Allows basic interaction with preview', () => {
    it('should handle node click events', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      fireEvent.click(triggerNode);
      
      expect(mockOnNodeClick).toHaveBeenCalledWith({
        id: 'trigger',
        type: 'trigger',
        label: 'Webhook Trigger',
        data: expect.any(Object),
      });
    });

    it('should show node details on hover', async () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const helpscoutNode = screen.getByTestId('workflow-node-helpscout');
      fireEvent.mouseEnter(helpscoutNode);
      
      await waitFor(() => {
        const tooltip = screen.getByTestId('node-tooltip-helpscout');
        expect(tooltip).toBeInTheDocument();
        expect(tooltip).toHaveTextContent('HelpScout Integration');
        expect(tooltip).toHaveTextContent('Fetches support tickets');
      });
      
      expect(mockOnNodeHover).toHaveBeenCalledWith('helpscout', true);
    });

    it('should allow clicking on connections', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const connection = screen.getByTestId('workflow-connection-trigger-helpscout');
      fireEvent.click(connection);
      
      expect(mockOnConnectionClick).toHaveBeenCalledWith({
        from: 'trigger',
        to: 'helpscout',
        label: 'Ticket Data',
      });
    });

    it('should support keyboard navigation between nodes', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      triggerNode.focus();
      
      fireEvent.keyDown(triggerNode, { key: 'ArrowRight' });
      
      const helpscoutNode = screen.getByTestId('workflow-node-helpscout');
      expect(document.activeElement).toBe(helpscoutNode);
    });

    it('should show context menu on right-click', async () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const slackNode = screen.getByTestId('workflow-node-slack');
      fireEvent.contextMenu(slackNode);
      
      await waitFor(() => {
        const contextMenu = screen.getByTestId('node-context-menu');
        expect(contextMenu).toBeInTheDocument();
        expect(screen.getByText('Edit Node')).toBeInTheDocument();
        expect(screen.getByText('Delete Node')).toBeInTheDocument();
        expect(screen.getByText('Duplicate Node')).toBeInTheDocument();
      });
    });

    it('should allow zooming and panning', () => {
      render(<WorkflowPreview {...defaultProps} enableZoomPan />);
      
      const canvas = screen.getByTestId('workflow-canvas');
      
      // Test zoom
      fireEvent.wheel(canvas, { deltaY: -100, ctrlKey: true });
      expect(canvas).toHaveAttribute('data-zoom', '1.1');
      
      // Test pan
      fireEvent.mouseDown(canvas, { clientX: 100, clientY: 100 });
      fireEvent.mouseMove(canvas, { clientX: 150, clientY: 150 });
      fireEvent.mouseUp(canvas);
      
      expect(canvas).toHaveAttribute('data-pan-x', '50');
      expect(canvas).toHaveAttribute('data-pan-y', '50');
    });

    it('should toggle between compact and expanded view', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const toggleButton = screen.getByTestId('view-toggle-button');
      expect(screen.getByTestId('workflow-container')).toHaveClass('expandedView');
      
      fireEvent.click(toggleButton);
      
      expect(screen.getByTestId('workflow-container')).toHaveClass('compactView');
      expect(toggleButton).toHaveTextContent('Expand View');
    });
  });

  describe('Integration with other components', () => {
    it('should work with DynamicForm field changes', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} />);
      
      const updatedIntent = {
        ...mockIntent,
        parameters: {
          ...mockIntent.parameters,
          triggerType: 'schedule',
          schedule: 'every hour',
        },
      };
      
      rerender(<WorkflowPreview {...defaultProps} intent={updatedIntent} />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      expect(triggerNode).toHaveTextContent('Schedule: Every hour');
    });

    it('should reflect WorkflowIntentAnalyzer updates', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} />);
      
      const modifyIntent: WorkflowIntent = {
        type: IntentType.MODIFY_WORKFLOW,
        confidence: 0.85,
        extractedEntities: {
          ...mockIntent.extractedEntities,
          modifications: [{
            action: 'add',
            component: 'notification',
            service: 'email',
          }],
        },
      };
      
      rerender(<WorkflowPreview {...defaultProps} intent={modifyIntent} />);
      
      expect(screen.getByTestId('workflow-node-email')).toBeInTheDocument();
      expect(screen.getByTestId('workflow-node-email')).toHaveClass('nodeNew');
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels for nodes', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      expect(triggerNode).toHaveAttribute('role', 'button');
      expect(triggerNode).toHaveAttribute('aria-label', 'Webhook Trigger node');
      expect(triggerNode).toHaveAttribute('tabindex', '0');
    });

    it('should announce state changes to screen readers', () => {
      const { rerender } = render(<WorkflowPreview {...defaultProps} currentStep="trigger" />);
      
      rerender(<WorkflowPreview {...defaultProps} currentStep="source" />);
      
      const announcement = screen.getByRole('status');
      expect(announcement).toHaveTextContent('Now configuring HelpScout step');
    });

    it('should support keyboard-only interaction', () => {
      render(<WorkflowPreview {...defaultProps} />);
      
      const triggerNode = screen.getByTestId('workflow-node-trigger');
      triggerNode.focus();
      
      fireEvent.keyDown(triggerNode, { key: 'Enter' });
      expect(mockOnNodeClick).toHaveBeenCalled();
      
      fireEvent.keyDown(triggerNode, { key: ' ' });
      expect(mockOnNodeClick).toHaveBeenCalledTimes(2);
    });
  });
});