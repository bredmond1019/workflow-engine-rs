export interface WorkflowIntent {
  type: string;
  confidence: number;
  suggestedFields?: FormField[];
  suggestedSteps?: FormStep[];
  extractedEntities: Record<string, any>;
  suggestedWorkflows?: string[];
  parameters?: Record<string, any>;
}

export interface FormField {
  name: string;
  type: 'text' | 'email' | 'tel' | 'number' | 'date' | 'time' | 'datetime-local' | 
        'select' | 'textarea' | 'checkbox' | 'radio';
  label: string;
  required: boolean;
  placeholder?: string;
  defaultValue?: any;
  options?: SelectOption[];
  validation?: FieldValidation;
}

export interface SelectOption {
  value: string;
  label: string;
}

export interface FieldValidation {
  min?: number;
  max?: number;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  message?: string;
}

export interface FormStep {
  step: number;
  name: string;
  fields: FormField[];
  description?: string;
}

export interface FormValues {
  [key: string]: any;
}

export interface FormErrors {
  [key: string]: string;
}