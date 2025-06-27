import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DynamicForm } from './DynamicForm';
import { ChatMessage } from '../../types/chat';
import { WorkflowIntent } from '../../types/workflow';

describe('DynamicForm', () => {
  const mockOnSubmit = jest.fn();
  const mockOnFieldChange = jest.fn();

  const sampleMessages: ChatMessage[] = [
    {
      id: '1',
      role: 'user',
      content: 'I need to create a contact form',
      timestamp: new Date('2024-01-01T10:00:00')
    },
    {
      id: '2',
      role: 'assistant',
      content: 'I can help you create a contact form. What fields would you like to include?',
      timestamp: new Date('2024-01-01T10:00:10')
    },
    {
      id: '3',
      role: 'user',
      content: 'Name, email, phone number, and a message field',
      timestamp: new Date('2024-01-01T10:00:20')
    }
  ];

  const sampleIntent: WorkflowIntent = {
    type: 'data_collection',
    confidence: 0.9,
    suggestedFields: [
      { name: 'name', type: 'text', label: 'Name', required: true },
      { name: 'email', type: 'email', label: 'Email', required: true },
      { name: 'phone', type: 'tel', label: 'Phone Number', required: false },
      { name: 'message', type: 'textarea', label: 'Message', required: true }
    ],
    extractedEntities: {
      formType: 'contact',
      fields: ['name', 'email', 'phone', 'message']
    }
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('5a. Creates text input from chat context', () => {
    it('should generate text input fields based on conversation', () => {
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const nameInput = screen.getByLabelText('Name');
      expect(nameInput).toBeInTheDocument();
      expect(nameInput).toHaveAttribute('type', 'text');
      expect(nameInput).toHaveAttribute('required');

      const phoneInput = screen.getByLabelText('Phone Number');
      expect(phoneInput).toBeInTheDocument();
      expect(phoneInput).toHaveAttribute('type', 'tel');
      expect(phoneInput).not.toHaveAttribute('required');
    });

    it('should handle different text field types (email, tel, textarea)', () => {
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const emailInput = screen.getByLabelText('Email');
      expect(emailInput).toHaveAttribute('type', 'email');

      const messageTextarea = screen.getByLabelText('Message');
      expect(messageTextarea.tagName).toBe('TEXTAREA');
    });

    it('should update form values when user types', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const nameInput = screen.getByLabelText('Name');
      await user.type(nameInput, 'John Doe');

      expect(mockOnFieldChange).toHaveBeenCalledWith('name', 'John Doe');
      expect(nameInput).toHaveValue('John Doe');
    });
  });

  describe('5b. Generates select dropdown from options', () => {
    const intentWithSelect: WorkflowIntent = {
      type: 'data_collection',
      confidence: 0.9,
      suggestedFields: [
        {
          name: 'category',
          type: 'select',
          label: 'Category',
          required: true,
          options: [
            { value: 'bug', label: 'Bug Report' },
            { value: 'feature', label: 'Feature Request' },
            { value: 'support', label: 'Support' }
          ]
        },
        {
          name: 'priority',
          type: 'select',
          label: 'Priority',
          required: false,
          options: [
            { value: 'low', label: 'Low' },
            { value: 'medium', label: 'Medium' },
            { value: 'high', label: 'High' }
          ]
        }
      ],
      extractedEntities: {}
    };

    it('should generate select dropdowns with options', () => {
      render(
        <DynamicForm
          messages={[]}
          intent={intentWithSelect}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const categorySelect = screen.getByLabelText('Category');
      expect(categorySelect).toBeInTheDocument();
      expect(categorySelect.tagName).toBe('SELECT');

      const options = screen.getAllByRole('option', { name: /Bug Report|Feature Request|Support/ });
      expect(options).toHaveLength(3);
    });

    it('should handle select field changes', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={[]}
          intent={intentWithSelect}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const categorySelect = screen.getByLabelText('Category');
      await user.selectOptions(categorySelect, 'feature');

      expect(mockOnFieldChange).toHaveBeenCalledWith('category', 'feature');
    });

    it('should show placeholder option for non-required selects', () => {
      render(
        <DynamicForm
          messages={[]}
          intent={intentWithSelect}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const prioritySelect = screen.getByLabelText('Priority');
      const placeholderOption = prioritySelect.querySelector('option[value=""]');
      expect(placeholderOption).toBeInTheDocument();
      expect(placeholderOption).toHaveTextContent('Select an option');
    });
  });

  describe('5c. Builds multi-step form progressively', () => {
    const multiStepIntent: WorkflowIntent = {
      type: 'multi_step_process',
      confidence: 0.95,
      suggestedSteps: [
        {
          step: 1,
          name: 'personal_info',
          fields: [
            { name: 'firstName', type: 'text', label: 'First Name', required: true },
            { name: 'lastName', type: 'text', label: 'Last Name', required: true }
          ]
        },
        {
          step: 2,
          name: 'contact_info',
          fields: [
            { name: 'email', type: 'email', label: 'Email', required: true },
            { name: 'phone', type: 'tel', label: 'Phone', required: false }
          ]
        },
        {
          step: 3,
          name: 'preferences',
          fields: [
            { 
              name: 'newsletter', 
              type: 'checkbox', 
              label: 'Subscribe to newsletter', 
              required: false 
            }
          ]
        }
      ],
      extractedEntities: {}
    };

    it('should display current step indicator', () => {
      render(
        <DynamicForm
          messages={[]}
          intent={multiStepIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          enableMultiStep={true}
        />
      );

      expect(screen.getByText('Step 1 of 3')).toBeInTheDocument();
      expect(screen.getByText('Personal Info')).toBeInTheDocument();
    });

    it('should show only current step fields', () => {
      render(
        <DynamicForm
          messages={[]}
          intent={multiStepIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          enableMultiStep={true}
        />
      );

      // Step 1 fields should be visible
      expect(screen.getByLabelText('First Name')).toBeInTheDocument();
      expect(screen.getByLabelText('Last Name')).toBeInTheDocument();

      // Step 2 fields should not be visible
      expect(screen.queryByLabelText('Email')).not.toBeInTheDocument();
      expect(screen.queryByLabelText('Phone')).not.toBeInTheDocument();
    });

    it('should navigate between steps', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={[]}
          intent={multiStepIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          enableMultiStep={true}
        />
      );

      // Fill required fields in step 1
      await user.type(screen.getByLabelText('First Name'), 'John');
      await user.type(screen.getByLabelText('Last Name'), 'Doe');

      // Click next
      const nextButton = screen.getByText('Next');
      await user.click(nextButton);

      // Should now show step 2
      expect(screen.getByText('Step 2 of 3')).toBeInTheDocument();
      expect(screen.getByLabelText('Email')).toBeInTheDocument();

      // Previous button should be visible
      expect(screen.getByText('Previous')).toBeInTheDocument();
    });

    it('should validate required fields before allowing next step', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={[]}
          intent={multiStepIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          enableMultiStep={true}
        />
      );

      // Try to go next without filling required fields
      const nextButton = screen.getByText('Next');
      await user.click(nextButton);

      // Should show validation errors
      expect(screen.getByText('First Name is required')).toBeInTheDocument();
      expect(screen.getByText('Last Name is required')).toBeInTheDocument();

      // Should still be on step 1
      expect(screen.getByText('Step 1 of 3')).toBeInTheDocument();
    });

    it('should show submit button on last step', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={[]}
          intent={multiStepIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          enableMultiStep={true}
          initialValues={{
            firstName: 'John',
            lastName: 'Doe',
            email: 'john@example.com'
          }}
        />
      );

      // Navigate to last step
      await user.click(screen.getByText('Next'));
      await user.click(screen.getByText('Next'));

      // Should show submit instead of next
      expect(screen.getByText('Submit')).toBeInTheDocument();
      expect(screen.queryByText('Next')).not.toBeInTheDocument();
    });
  });

  describe('5d. Validates fields through conversation', () => {
    const validationMessages: ChatMessage[] = [
      ...sampleMessages,
      {
        id: '4',
        role: 'assistant',
        content: 'The email field should validate email format, and phone should accept international formats',
        timestamp: new Date('2024-01-01T10:00:30')
      }
    ];

    it('should validate email format', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={validationMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const emailInput = screen.getByLabelText('Email');
      await user.type(emailInput, 'invalid-email');
      await user.tab(); // Trigger blur

      expect(screen.getByText('Please enter a valid email address')).toBeInTheDocument();
    });

    it('should validate required fields', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={validationMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const nameInput = screen.getByLabelText('Name');
      await user.click(nameInput);
      await user.tab(); // Blur without typing

      expect(screen.getByText('Name is required')).toBeInTheDocument();
    });

    it('should show custom validation messages from intent', () => {
      const intentWithValidation: WorkflowIntent = {
        ...sampleIntent,
        suggestedFields: [
          {
            name: 'age',
            type: 'number',
            label: 'Age',
            required: true,
            validation: {
              min: 18,
              max: 100,
              message: 'Age must be between 18 and 100'
            }
          }
        ]
      };

      render(
        <DynamicForm
          messages={[]}
          intent={intentWithValidation}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const ageInput = screen.getByLabelText('Age');
      fireEvent.change(ageInput, { target: { value: '15' } });
      fireEvent.blur(ageInput);

      expect(screen.getByText('Age must be between 18 and 100')).toBeInTheDocument();
    });

    it('should prevent form submission with validation errors', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={validationMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const submitButton = screen.getByText('Submit');
      await user.click(submitButton);

      expect(mockOnSubmit).not.toHaveBeenCalled();
      expect(screen.getByText('Please fix the errors before submitting')).toBeInTheDocument();
    });

    it('should clear validation errors when corrected', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={validationMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const emailInput = screen.getByLabelText('Email');
      
      // Type invalid email
      await user.type(emailInput, 'invalid');
      await user.tab();
      expect(screen.getByText('Please enter a valid email address')).toBeInTheDocument();

      // Clear and type valid email
      await user.clear(emailInput);
      await user.type(emailInput, 'valid@example.com');
      await user.tab();
      
      expect(screen.queryByText('Please enter a valid email address')).not.toBeInTheDocument();
    });
  });

  describe('5e. Shows form preview alongside chat', () => {
    it('should render form preview section', () => {
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          showPreview={true}
        />
      );

      expect(screen.getByText('Form Preview')).toBeInTheDocument();
      expect(screen.getByTestId('form-preview')).toBeInTheDocument();
    });

    it('should update preview in real-time as user types', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          showPreview={true}
        />
      );

      const nameInput = screen.getByLabelText('Name');
      await user.type(nameInput, 'John Doe');

      const preview = screen.getByTestId('form-preview');
      expect(preview).toHaveTextContent('Name: John Doe');
    });

    it('should show preview with formatted values', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          showPreview={true}
        />
      );

      // Fill multiple fields
      await user.type(screen.getByLabelText('Name'), 'John Doe');
      await user.type(screen.getByLabelText('Email'), 'john@example.com');
      await user.type(screen.getByLabelText('Phone Number'), '+1 (555) 123-4567');

      const preview = screen.getByTestId('form-preview');
      expect(preview).toHaveTextContent('Name: John Doe');
      expect(preview).toHaveTextContent('Email: john@example.com');
      expect(preview).toHaveTextContent('Phone Number: +1 (555) 123-4567');
    });

    it('should indicate required fields in preview', () => {
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          showPreview={true}
        />
      );

      const preview = screen.getByTestId('form-preview');
      const requiredFields = preview.querySelectorAll('[data-required="true"]');
      
      // Name, Email, and Message are required
      expect(requiredFields).toHaveLength(3);
    });

    it('should show preview summary with completion status', () => {
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          showPreview={true}
          initialValues={{
            name: 'John Doe',
            email: 'john@example.com'
          }}
        />
      );

      expect(screen.getByText('2 of 4 fields completed')).toBeInTheDocument();
      expect(screen.getByText('50% complete')).toBeInTheDocument();
    });

    it('should toggle preview visibility', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          showPreview={true}
        />
      );

      const toggleButton = screen.getByLabelText('Toggle preview');
      const preview = screen.getByTestId('form-preview');

      expect(preview).toBeVisible();

      await user.click(toggleButton);
      expect(preview).not.toBeVisible();

      await user.click(toggleButton);
      expect(preview).toBeVisible();
    });
  });

  describe('Additional features', () => {
    it('should support date and time fields', () => {
      const dateIntent: WorkflowIntent = {
        type: 'scheduling',
        confidence: 0.9,
        suggestedFields: [
          { name: 'date', type: 'date', label: 'Date', required: true },
          { name: 'time', type: 'time', label: 'Time', required: true },
          { name: 'datetime', type: 'datetime-local', label: 'Date & Time', required: false }
        ],
        extractedEntities: {}
      };

      render(
        <DynamicForm
          messages={[]}
          intent={dateIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      expect(screen.getByLabelText('Date')).toHaveAttribute('type', 'date');
      expect(screen.getByLabelText('Time')).toHaveAttribute('type', 'time');
      expect(screen.getByLabelText('Date & Time')).toHaveAttribute('type', 'datetime-local');
    });

    it('should support checkbox fields', async () => {
      const user = userEvent.setup();
      const checkboxIntent: WorkflowIntent = {
        type: 'preferences',
        confidence: 0.9,
        suggestedFields: [
          { name: 'terms', type: 'checkbox', label: 'I agree to terms', required: true },
          { name: 'newsletter', type: 'checkbox', label: 'Subscribe to newsletter', required: false }
        ],
        extractedEntities: {}
      };

      render(
        <DynamicForm
          messages={[]}
          intent={checkboxIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const termsCheckbox = screen.getByLabelText('I agree to terms');
      expect(termsCheckbox).toHaveAttribute('type', 'checkbox');

      await user.click(termsCheckbox);
      expect(mockOnFieldChange).toHaveBeenCalledWith('terms', true);
    });

    it('should support radio button groups', async () => {
      const user = userEvent.setup();
      const radioIntent: WorkflowIntent = {
        type: 'survey',
        confidence: 0.9,
        suggestedFields: [
          {
            name: 'experience',
            type: 'radio',
            label: 'How was your experience?',
            required: true,
            options: [
              { value: 'excellent', label: 'Excellent' },
              { value: 'good', label: 'Good' },
              { value: 'poor', label: 'Poor' }
            ]
          }
        ],
        extractedEntities: {}
      };

      render(
        <DynamicForm
          messages={[]}
          intent={radioIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      const excellentRadio = screen.getByLabelText('Excellent');
      await user.click(excellentRadio);

      expect(mockOnFieldChange).toHaveBeenCalledWith('experience', 'excellent');
    });

    it('should handle form submission with all values', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
        />
      );

      // Fill all fields
      await user.type(screen.getByLabelText('Name'), 'John Doe');
      await user.type(screen.getByLabelText('Email'), 'john@example.com');
      await user.type(screen.getByLabelText('Phone Number'), '+1234567890');
      await user.type(screen.getByLabelText('Message'), 'This is a test message');

      // Submit form
      await user.click(screen.getByText('Submit'));

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalledWith({
          name: 'John Doe',
          email: 'john@example.com',
          phone: '+1234567890',
          message: 'This is a test message'
        });
      });
    });

    it('should reset form after submission', async () => {
      const user = userEvent.setup();
      render(
        <DynamicForm
          messages={sampleMessages}
          intent={sampleIntent}
          onSubmit={mockOnSubmit}
          onFieldChange={mockOnFieldChange}
          resetOnSubmit={true}
        />
      );

      // Fill and submit
      await user.type(screen.getByLabelText('Name'), 'John Doe');
      await user.type(screen.getByLabelText('Email'), 'john@example.com');
      await user.type(screen.getByLabelText('Message'), 'Test');
      
      await user.click(screen.getByText('Submit'));

      await waitFor(() => {
        expect(screen.getByLabelText('Name')).toHaveValue('');
        expect(screen.getByLabelText('Email')).toHaveValue('');
        expect(screen.getByLabelText('Message')).toHaveValue('');
      });
    });
  });
});