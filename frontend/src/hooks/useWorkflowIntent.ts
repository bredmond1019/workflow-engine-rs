import { useState, useCallback, useRef } from 'react';
import { WorkflowIntentAnalyzer, WorkflowIntent } from '../services/ai/WorkflowIntentAnalyzer';

interface UseWorkflowIntentReturn {
  currentIntent: WorkflowIntent | null;
  isAnalyzing: boolean;
  error: string | null;
  analyzeMessage: (message: string) => Promise<void>;
  clearIntent: () => void;
  intentHistory: WorkflowIntent[];
}

export function useWorkflowIntent(): UseWorkflowIntentReturn {
  const [currentIntent, setCurrentIntent] = useState<WorkflowIntent | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [intentHistory, setIntentHistory] = useState<WorkflowIntent[]>([]);
  
  const analyzerRef = useRef<WorkflowIntentAnalyzer>(new WorkflowIntentAnalyzer());
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const analyzeMessage = useCallback(async (message: string) => {
    // Clear any existing debounce timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    // Debounce the analysis
    return new Promise<void>((resolve) => {
      debounceTimerRef.current = setTimeout(async () => {
        setIsAnalyzing(true);
        setError(null);

        try {
          const intent = await analyzerRef.current.analyzeIntent(message);
          setCurrentIntent(intent);
          setIntentHistory(prev => [...prev, intent]);
          resolve();
        } catch (err) {
          setError('Failed to analyze message');
          console.error('Intent analysis error:', err);
          resolve();
        } finally {
          setIsAnalyzing(false);
        }
      }, 300); // 300ms debounce delay
    });
  }, []);

  const clearIntent = useCallback(() => {
    setCurrentIntent(null);
    setError(null);
    analyzerRef.current.clearContext();
  }, []);

  return {
    currentIntent,
    isAnalyzing,
    error,
    analyzeMessage,
    clearIntent,
    intentHistory
  };
}