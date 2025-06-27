import { renderHook, act } from '@testing-library/react';
import { useWorkflowIntent } from './useWorkflowIntent';
import { WorkflowIntentAnalyzer, IntentType, WorkflowType } from '../services/ai/WorkflowIntentAnalyzer';

// Mock the WorkflowIntentAnalyzer
jest.mock('../services/ai/WorkflowIntentAnalyzer');

describe('useWorkflowIntent Hook', () => {
  let mockAnalyzer: jest.Mocked<WorkflowIntentAnalyzer>;

  beforeEach(() => {
    jest.clearAllMocks();
    mockAnalyzer = new WorkflowIntentAnalyzer() as jest.Mocked<WorkflowIntentAnalyzer>;
    (WorkflowIntentAnalyzer as jest.Mock).mockImplementation(() => mockAnalyzer);
  });

  it('should initialize with no intent', () => {
    const { result } = renderHook(() => useWorkflowIntent());

    expect(result.current.currentIntent).toBeNull();
    expect(result.current.isAnalyzing).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('should analyze intent when called', async () => {
    const mockIntent = {
      type: IntentType.CREATE_WORKFLOW,
      confidence: 0.9,
      workflowType: WorkflowType.CUSTOMER_SUPPORT,
      parameters: [],
      needsClarification: false
    };

    mockAnalyzer.analyzeIntent.mockResolvedValue(mockIntent);

    const { result } = renderHook(() => useWorkflowIntent());

    await act(async () => {
      await result.current.analyzeMessage('Create a customer support workflow');
    });

    expect(result.current.currentIntent).toEqual(mockIntent);
    expect(result.current.isAnalyzing).toBe(false);
    expect(mockAnalyzer.analyzeIntent).toHaveBeenCalledWith('Create a customer support workflow');
  });

  it('should handle analysis errors', async () => {
    const error = new Error('Analysis failed');
    mockAnalyzer.analyzeIntent.mockRejectedValue(error);

    const { result } = renderHook(() => useWorkflowIntent());

    await act(async () => {
      await result.current.analyzeMessage('Invalid message');
    });

    expect(result.current.currentIntent).toBeNull();
    expect(result.current.error).toBe('Failed to analyze message');
    expect(result.current.isAnalyzing).toBe(false);
  });

  it('should show loading state during analysis', async () => {
    let resolvePromise: (value: any) => void;
    const promise = new Promise((resolve) => {
      resolvePromise = resolve;
    });

    mockAnalyzer.analyzeIntent.mockReturnValue(promise as any);

    const { result } = renderHook(() => useWorkflowIntent());

    act(() => {
      result.current.analyzeMessage('Test message');
    });

    // Wait for debounce
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 350));
    });

    expect(result.current.isAnalyzing).toBe(true);

    await act(async () => {
      resolvePromise!({
        type: IntentType.CREATE_WORKFLOW,
        confidence: 0.8,
        parameters: [],
        needsClarification: false
      });
    });

    expect(result.current.isAnalyzing).toBe(false);
  });

  it('should clear intent when requested', async () => {
    const mockIntent = {
      type: IntentType.CREATE_WORKFLOW,
      confidence: 0.9,
      parameters: [],
      needsClarification: false
    };

    mockAnalyzer.analyzeIntent.mockResolvedValue(mockIntent);

    const { result } = renderHook(() => useWorkflowIntent());

    await act(async () => {
      await result.current.analyzeMessage('Create workflow');
    });

    expect(result.current.currentIntent).toBeTruthy();

    act(() => {
      result.current.clearIntent();
    });

    expect(result.current.currentIntent).toBeNull();
    expect(mockAnalyzer.clearContext).toHaveBeenCalled();
  });

  it('should handle clarification requests', async () => {
    const mockIntent = {
      type: IntentType.CREATE_WORKFLOW,
      confidence: 0.4,
      parameters: [],
      needsClarification: true,
      clarificationQuestions: ['What type of workflow?', 'When should it trigger?']
    };

    mockAnalyzer.analyzeIntent.mockResolvedValue(mockIntent);

    const { result } = renderHook(() => useWorkflowIntent());

    await act(async () => {
      await result.current.analyzeMessage('I want to create something');
    });

    expect(result.current.currentIntent?.needsClarification).toBe(true);
    expect(result.current.currentIntent?.clarificationQuestions).toHaveLength(2);
  });

  it('should provide intent suggestions', async () => {
    const mockIntent = {
      type: IntentType.CREATE_WORKFLOW,
      confidence: 0.6,
      workflowType: WorkflowType.GENERIC,
      parameters: [],
      needsClarification: true,
      suggestions: [
        { template: 'customer-support', description: 'Handle customer tickets' },
        { template: 'data-sync', description: 'Sync data between systems' }
      ]
    };

    mockAnalyzer.analyzeIntent.mockResolvedValue(mockIntent);

    const { result } = renderHook(() => useWorkflowIntent());

    await act(async () => {
      await result.current.analyzeMessage('Help me automate');
    });

    expect(result.current.currentIntent?.suggestions).toHaveLength(2);
    expect(result.current.currentIntent?.suggestions?.[0].template).toBe('customer-support');
  });

  it('should debounce rapid analysis calls', async () => {
    mockAnalyzer.analyzeIntent.mockResolvedValue({
      type: IntentType.CREATE_WORKFLOW,
      confidence: 0.8,
      parameters: [],
      needsClarification: false
    });

    const { result } = renderHook(() => useWorkflowIntent());

    // Make multiple rapid calls
    act(() => {
      result.current.analyzeMessage('First');
      result.current.analyzeMessage('Second');
      result.current.analyzeMessage('Third');
    });

    // Wait for debounce
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 500));
    });

    // Should only have analyzed the last message
    expect(mockAnalyzer.analyzeIntent).toHaveBeenCalledTimes(1);
    expect(mockAnalyzer.analyzeIntent).toHaveBeenCalledWith('Third');
  });

  it('should preserve analysis history', async () => {
    const mockIntents = [
      {
        type: IntentType.CREATE_WORKFLOW,
        confidence: 0.9,
        parameters: [],
        needsClarification: false
      },
      {
        type: IntentType.MODIFY_WORKFLOW,
        confidence: 0.8,
        parameters: [],
        needsClarification: false
      }
    ];

    mockAnalyzer.analyzeIntent
      .mockResolvedValueOnce(mockIntents[0])
      .mockResolvedValueOnce(mockIntents[1]);

    const { result } = renderHook(() => useWorkflowIntent());

    await act(async () => {
      await result.current.analyzeMessage('Create workflow');
    });

    await act(async () => {
      await result.current.analyzeMessage('Modify it');
    });

    expect(result.current.intentHistory).toHaveLength(2);
    expect(result.current.intentHistory[0].type).toBe(IntentType.CREATE_WORKFLOW);
    expect(result.current.intentHistory[1].type).toBe(IntentType.MODIFY_WORKFLOW);
  });
});