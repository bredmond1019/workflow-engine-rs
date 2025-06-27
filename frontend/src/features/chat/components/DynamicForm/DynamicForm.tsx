import React from 'react';
import { ChatMessage } from '../../types/chat';
import { WorkflowIntent } from '../../types/workflow';
import styles from './DynamicForm.module.css';

interface DynamicFormProps {
  messages: ChatMessage[];
  intent: WorkflowIntent;
  onSubmit: (values: Record<string, any>) => void;
  onFieldChange: (fieldName: string, value: any) => void;
  showPreview?: boolean;
  enableMultiStep?: boolean;
  initialValues?: Record<string, any>;
  resetOnSubmit?: boolean;
}

export const DynamicForm: React.FC<DynamicFormProps> = ({
  messages,
  intent,
  onSubmit,
  onFieldChange,
  showPreview = false,
  enableMultiStep = false,
  initialValues = {},
  resetOnSubmit = false
}) => {
  // Minimal implementation that will fail all tests
  return (
    <div className={styles.dynamicForm}>
      <div className={styles.formContainer}>
        {/* Form will be implemented here */}
      </div>
    </div>
  );
};