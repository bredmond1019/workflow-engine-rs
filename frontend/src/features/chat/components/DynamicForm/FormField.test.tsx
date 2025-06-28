import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { FormField } from './FormField';
import { FormField as FormFieldType } from '../../types/workflow';

describe('FormField', () => {
  const mockOnChange = jest.fn();
  const mockOnBlur = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders text input correctly', () => {
    const field: FormFieldType = {
      name: 'title',
      label: 'Title',
      type: 'text',
      required: true
    };

    render(
      <FormField
        field={field}
        value="test value"
        onChange={mockOnChange}
        onBlur={mockOnBlur}
      />
    );

    expect(screen.getByLabelText('Title')).toBeInTheDocument();
    expect(screen.getByDisplayValue('test value')).toBeInTheDocument();
    expect(screen.getByText('*')).toBeInTheDocument();
  });

  it('renders select field with options', () => {
    const field: FormFieldType = {
      name: 'category',
      label: 'Category',
      type: 'select',
      required: false,
      options: [
        { value: 'option1', label: 'Option 1' },
        { value: 'option2', label: 'Option 2' }
      ]
    };

    render(
      <FormField
        field={field}
        value="option1"
        onChange={mockOnChange}
        onBlur={mockOnBlur}
      />
    );

    expect(screen.getByLabelText('Category')).toBeInTheDocument();
    const select = screen.getByRole('combobox');
    expect(select).toHaveValue('option1');
    expect(screen.getByText('Option 1')).toBeInTheDocument();
    expect(screen.getByText('Option 2')).toBeInTheDocument();
  });

  it('handles change events correctly', () => {
    const field: FormFieldType = {
      name: 'title',
      label: 'Title',
      type: 'text',
      required: true
    };

    render(
      <FormField
        field={field}
        value=""
        onChange={mockOnChange}
        onBlur={mockOnBlur}
      />
    );

    const input = screen.getByLabelText('Title');
    fireEvent.change(input, { target: { value: 'new value' } });

    expect(mockOnChange).toHaveBeenCalledWith('title', 'new value');
  });

  it('handles blur events correctly', () => {
    const field: FormFieldType = {
      name: 'title',
      label: 'Title',
      type: 'text',
      required: true
    };

    render(
      <FormField
        field={field}
        value="test"
        onChange={mockOnChange}
        onBlur={mockOnBlur}
      />
    );

    const input = screen.getByLabelText('Title');
    fireEvent.blur(input);

    expect(mockOnBlur).toHaveBeenCalledWith('title');
  });

  it('displays error message when provided', () => {
    const field: FormFieldType = {
      name: 'email',
      label: 'Email',
      type: 'email',
      required: true
    };

    render(
      <FormField
        field={field}
        value="invalid-email"
        error="Please enter a valid email address"
        onChange={mockOnChange}
        onBlur={mockOnBlur}
      />
    );

    expect(screen.getByText('Please enter a valid email address')).toBeInTheDocument();
  });

  it('renders checkbox field correctly', () => {
    const field: FormFieldType = {
      name: 'agree',
      label: 'I agree to terms',
      type: 'checkbox',
      required: true
    };

    render(
      <FormField
        field={field}
        value={true}
        onChange={mockOnChange}
        onBlur={mockOnBlur}
      />
    );

    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeChecked();
    expect(screen.getByText('I agree to terms')).toBeInTheDocument();
  });
});