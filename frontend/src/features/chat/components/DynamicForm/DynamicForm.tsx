import React, { useState } from 'react';
import { ChatMessage } from '../../types/chat';
import { WorkflowIntent, FormField, FormValues, FormErrors } from '../../types/workflow';
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
  const [values, setValues] = useState<FormValues>(initialValues);
  const [errors, setErrors] = useState<FormErrors>({});
  const [touched, setTouched] = useState<Record<string, boolean>>({});
  const [currentStep, setCurrentStep] = useState(1);
  const [showPreviewState, setShowPreviewState] = useState(showPreview);
  const [hasTriedSubmit, setHasTriedSubmit] = useState(false);

  // Get fields based on whether we're in multi-step mode
  const getFields = (): FormField[] => {
    if (enableMultiStep && intent.suggestedSteps) {
      const currentStepData = intent.suggestedSteps.find(s => s.step === currentStep);
      return currentStepData?.fields || [];
    }
    return intent.suggestedFields || [];
  };

  // Get total steps
  const totalSteps = enableMultiStep && intent.suggestedSteps ? intent.suggestedSteps.length : 1;

  // Get current step name
  const getCurrentStepName = (): string => {
    if (enableMultiStep && intent.suggestedSteps) {
      const currentStepData = intent.suggestedSteps.find(s => s.step === currentStep);
      return currentStepData?.name?.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()) || '';
    }
    return '';
  };

  // Validate email
  const isValidEmail = (email: string): boolean => {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  };

  // Validate field
  const validateField = (field: FormField, value: any): string | null => {
    if (field.required && (!value || value === '')) {
      return `${field.label} is required`;
    }

    if (field.type === 'email' && value && !isValidEmail(value)) {
      return 'Please enter a valid email address';
    }

    if (field.validation) {
      const { min, max, message } = field.validation;
      if (field.type === 'number') {
        const numValue = Number(value);
        if ((min !== undefined && numValue < min) || (max !== undefined && numValue > max)) {
          return message || `Value must be between ${min} and ${max}`;
        }
      }
    }

    return null;
  };

  // Handle field change
  const handleFieldChange = (fieldName: string, value: any) => {
    setValues(prev => ({ ...prev, [fieldName]: value }));
    onFieldChange(fieldName, value);

    // Validate on change if field was touched
    if (touched[fieldName]) {
      const field = getFields().find(f => f.name === fieldName);
      if (field) {
        const error = validateField(field, value);
        setErrors(prev => ({
          ...prev,
          [fieldName]: error || ''
        }));
      }
    }
  };

  // Handle blur
  const handleBlur = (fieldName: string) => {
    setTouched(prev => ({ ...prev, [fieldName]: true }));
    const field = getFields().find(f => f.name === fieldName);
    if (field) {
      const error = validateField(field, values[fieldName]);
      setErrors(prev => ({
        ...prev,
        [fieldName]: error || ''
      }));
    }
  };

  // Validate all fields in current step
  const validateCurrentStep = (): boolean => {
    const fields = getFields();
    const newErrors: FormErrors = {};
    let isValid = true;

    fields.forEach(field => {
      const error = validateField(field, values[field.name]);
      if (error) {
        newErrors[field.name] = error;
        isValid = false;
      }
    });

    setErrors(prev => ({ ...prev, ...newErrors }));
    setTouched(prev => {
      const newTouched = { ...prev };
      fields.forEach(field => {
        newTouched[field.name] = true;
      });
      return newTouched;
    });

    return isValid;
  };

  // Handle next step
  const handleNext = () => {
    if (validateCurrentStep()) {
      setCurrentStep(prev => Math.min(prev + 1, totalSteps));
    }
  };

  // Handle previous step
  const handlePrevious = () => {
    setCurrentStep(prev => Math.max(prev - 1, 1));
  };

  // Handle form submission
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setHasTriedSubmit(true);

    // Validate all fields
    let allValid = true;
    const allErrors: FormErrors = {};
    const allTouched: Record<string, boolean> = {};

    const fieldsToValidate = enableMultiStep && intent.suggestedSteps
      ? intent.suggestedSteps.flatMap(s => s.fields)
      : intent.suggestedFields || [];

    fieldsToValidate.forEach(field => {
      const error = validateField(field, values[field.name]);
      if (error) {
        allErrors[field.name] = error;
        allValid = false;
      }
      allTouched[field.name] = true;
    });

    if (!allValid) {
      setErrors(allErrors);
      setTouched(allTouched);
      return;
    }

    onSubmit(values);

    if (resetOnSubmit) {
      setValues({});
      setErrors({});
      setTouched({});
      setCurrentStep(1);
      setHasTriedSubmit(false);
    }
  };

  // Render field
  const renderField = (field: FormField) => {
    const error = errors[field.name];
    const value = values[field.name] || '';

    switch (field.type) {
      case 'select':
        return (
          <div key={field.name} className={styles.fieldGroup}>
            <label htmlFor={field.name} className={styles.fieldLabel}>
              {field.label}
              {field.required && <span className={styles.required}>*</span>}
            </label>
            <select
              id={field.name}
              name={field.name}
              value={value}
              onChange={(e) => handleFieldChange(field.name, e.target.value)}
              onBlur={() => handleBlur(field.name)}
              required={field.required}
              className={`${styles.fieldInput} ${styles.fieldSelect}`}
              aria-label={field.label}
            >
              {!field.required && <option value="">Select an option</option>}
              {field.options?.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            {error && <span className={styles.fieldError}>{error}</span>}
          </div>
        );

      case 'textarea':
        return (
          <div key={field.name} className={styles.fieldGroup}>
            <label htmlFor={field.name} className={styles.fieldLabel}>
              {field.label}
              {field.required && <span className={styles.required}>*</span>}
            </label>
            <textarea
              id={field.name}
              name={field.name}
              value={value}
              onChange={(e) => handleFieldChange(field.name, e.target.value)}
              onBlur={() => handleBlur(field.name)}
              required={field.required}
              className={`${styles.fieldInput} ${styles.fieldTextarea}`}
              aria-label={field.label}
            />
            {error && <span className={styles.fieldError}>{error}</span>}
          </div>
        );

      case 'checkbox':
        return (
          <div key={field.name} className={styles.fieldGroup}>
            <div className={styles.checkboxGroup}>
              <input
                type="checkbox"
                id={field.name}
                name={field.name}
                checked={value || false}
                onChange={(e) => handleFieldChange(field.name, e.target.checked)}
                required={field.required}
                className={styles.checkboxInput}
                aria-label={field.label}
              />
              <label htmlFor={field.name} className={styles.checkboxLabel}>
                {field.label}
                {field.required && <span className={styles.required}>*</span>}
              </label>
            </div>
            {error && <span className={styles.fieldError}>{error}</span>}
          </div>
        );

      case 'radio':
        return (
          <div key={field.name} className={styles.fieldGroup}>
            <label className={styles.fieldLabel}>
              {field.label}
              {field.required && <span className={styles.required}>*</span>}
            </label>
            <div className={styles.radioOptions}>
              {field.options?.map(option => (
                <div key={option.value} className={styles.radioGroup}>
                  <input
                    type="radio"
                    id={`${field.name}-${option.value}`}
                    name={field.name}
                    value={option.value}
                    checked={value === option.value}
                    onChange={(e) => handleFieldChange(field.name, e.target.value)}
                    required={field.required}
                    className={styles.radioInput}
                    aria-label={option.label}
                  />
                  <label htmlFor={`${field.name}-${option.value}`} className={styles.radioLabel}>
                    {option.label}
                  </label>
                </div>
              ))}
            </div>
            {error && <span className={styles.fieldError}>{error}</span>}
          </div>
        );

      default:
        return (
          <div key={field.name} className={styles.fieldGroup}>
            <label htmlFor={field.name} className={styles.fieldLabel}>
              {field.label}
              {field.required && <span className={styles.required}>*</span>}
            </label>
            <input
              type={field.type}
              id={field.name}
              name={field.name}
              value={value}
              onChange={(e) => handleFieldChange(field.name, e.target.value)}
              onBlur={() => handleBlur(field.name)}
              required={field.required}
              className={styles.fieldInput}
              aria-label={field.label}
            />
            {error && <span className={styles.fieldError}>{error}</span>}
          </div>
        );
    }
  };

  // Calculate completion stats
  const getCompletionStats = () => {
    const allFields = enableMultiStep && intent.suggestedSteps
      ? intent.suggestedSteps.flatMap(s => s.fields)
      : intent.suggestedFields || [];
    
    const completedCount = allFields.filter(field => 
      values[field.name] !== undefined && values[field.name] !== ''
    ).length;
    
    const totalCount = allFields.length;
    const percentage = totalCount > 0 ? Math.round((completedCount / totalCount) * 100) : 0;
    
    return { completedCount, totalCount, percentage };
  };

  // Check if there are any errors
  const hasErrors = Object.values(errors).some(error => error && error.trim() !== '');
  
  // For form submission error display, also check if any required fields are empty
  const hasValidationErrors = hasErrors || (hasTriedSubmit && (
    (enableMultiStep && intent.suggestedSteps
      ? intent.suggestedSteps.flatMap(s => s.fields)
      : intent.suggestedFields || []
    ).some(field => 
      field.required && (!values[field.name] || values[field.name] === '')
    )
  ));

  return (
    <div className={styles.dynamicForm}>
      <form onSubmit={handleSubmit} className={styles.formContainer}>
        {enableMultiStep && totalSteps > 1 && (
          <div className={styles.stepIndicator}>
            <span>Step {currentStep} of {totalSteps}</span>
            {getCurrentStepName() && <span className={styles.stepName}>{getCurrentStepName()}</span>}
          </div>
        )}

        <div className={styles.formFields}>
          {getFields().map(field => renderField(field))}
        </div>

        {hasValidationErrors && (
          <div className={styles.formError}>
            Please fix the errors before submitting
          </div>
        )}

        <div className={styles.formActions}>
          {enableMultiStep && currentStep > 1 && (
            <button 
              type="button" 
              onClick={handlePrevious}
              className={`${styles.formButton} ${styles.secondaryButton}`}
            >
              Previous
            </button>
          )}
          
          {enableMultiStep && currentStep < totalSteps ? (
            <button 
              type="button" 
              onClick={handleNext}
              className={`${styles.formButton} ${styles.primaryButton}`}
            >
              Next
            </button>
          ) : (
            <button 
              type="button"
              className={`${styles.formButton} ${styles.primaryButton}`}
              onClick={(e) => {
                e.preventDefault();
                handleSubmit(e as any);
              }}
            >
              Submit
            </button>
          )}
        </div>
      </form>

      {showPreview && (
        <>
          <div className={styles.previewContainer}>
            <div className={styles.previewHeader}>
              <h3 className={styles.previewTitle}>Form Preview</h3>
              <button
                type="button"
                aria-label="Toggle preview"
                onClick={() => setShowPreviewState(!showPreviewState)}
                className={styles.toggleButton}
              >
                {showPreviewState ? '−' : '+'}
              </button>
            </div>

            <div 
              data-testid="form-preview" 
              className={styles.previewContent}
              style={{ display: showPreviewState ? 'block' : 'none' }}
            >
              {(enableMultiStep && intent.suggestedSteps
                ? intent.suggestedSteps.flatMap(s => s.fields)
                : intent.suggestedFields || []
              ).map(field => (
                <div 
                  key={field.name} 
                  className={styles.previewField}
                  data-required={field.required}
                >
                  <div>
                    <span className={styles.previewFieldLabel}>
                      {field.label}:&nbsp;
                    </span>
                    <span className={styles.previewFieldValue}>
                      {values[field.name] !== undefined && values[field.name] !== '' 
                        ? String(values[field.name]) 
                        : '(empty)'}
                    </span>
                    {field.required && <span className={styles.required} aria-hidden="true"> *</span>}
                  </div>
                </div>
              ))}

              <div className={styles.previewSummary}>
                {(() => {
                  const stats = getCompletionStats();
                  return (
                    <>
                      <div className={styles.previewProgress}>
                        {stats.completedCount} of {stats.totalCount} fields completed
                      </div>
                      <div className={styles.previewProgress}>
                        {stats.percentage}% complete
                      </div>
                      <div className={styles.progressBar}>
                        <div 
                          className={styles.progressFill}
                          style={{ width: `${stats.percentage}%` }}
                        />
                      </div>
                    </>
                  );
                })()}
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
};