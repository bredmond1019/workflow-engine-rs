import React from 'react';
import { FormField as FormFieldType } from '../../types/workflow';
import styles from './DynamicForm.module.css';

interface FormFieldProps {
  field: FormFieldType;
  value: any;
  error?: string;
  onChange: (fieldName: string, value: any) => void;
  onBlur: (fieldName: string) => void;
}

export const FormField: React.FC<FormFieldProps> = ({
  field,
  value,
  error,
  onChange,
  onBlur
}) => {
  const handleChange = (newValue: any) => {
    onChange(field.name, newValue);
  };

  const handleBlur = () => {
    onBlur(field.name);
  };

  const renderFieldLabel = () => (
    <label htmlFor={field.name} className={styles.fieldLabel}>
      {field.label}
      {field.required && <span className={styles.required}>*</span>}
    </label>
  );

  const renderError = () => 
    error && <span className={styles.fieldError}>{error}</span>;

  switch (field.type) {
    case 'select':
      return (
        <div className={styles.fieldGroup}>
          {renderFieldLabel()}
          <select
            id={field.name}
            name={field.name}
            value={value || ''}
            onChange={(e) => handleChange(e.target.value)}
            onBlur={handleBlur}
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
          {renderError()}
        </div>
      );

    case 'textarea':
      return (
        <div className={styles.fieldGroup}>
          {renderFieldLabel()}
          <textarea
            id={field.name}
            name={field.name}
            value={value || ''}
            onChange={(e) => handleChange(e.target.value)}
            onBlur={handleBlur}
            required={field.required}
            className={`${styles.fieldInput} ${styles.fieldTextarea}`}
            aria-label={field.label}
          />
          {renderError()}
        </div>
      );

    case 'checkbox':
      return (
        <div className={styles.fieldGroup}>
          <div className={styles.checkboxGroup}>
            <input
              type="checkbox"
              id={field.name}
              name={field.name}
              checked={value || false}
              onChange={(e) => handleChange(e.target.checked)}
              required={field.required}
              className={styles.checkboxInput}
              aria-label={field.label}
            />
            <label htmlFor={field.name} className={styles.checkboxLabel}>
              {field.label}
              {field.required && <span className={styles.required}>*</span>}
            </label>
          </div>
          {renderError()}
        </div>
      );

    case 'radio':
      return (
        <div className={styles.fieldGroup}>
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
                  onChange={(e) => handleChange(e.target.value)}
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
          {renderError()}
        </div>
      );

    default:
      return (
        <div className={styles.fieldGroup}>
          {renderFieldLabel()}
          <input
            type={field.type}
            id={field.name}
            name={field.name}
            value={value || ''}
            onChange={(e) => handleChange(e.target.value)}
            onBlur={handleBlur}
            required={field.required}
            className={styles.fieldInput}
            aria-label={field.label}
          />
          {renderError()}
        </div>
      );
  }
};