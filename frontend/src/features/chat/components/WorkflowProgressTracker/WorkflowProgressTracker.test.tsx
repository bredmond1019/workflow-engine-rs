import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { WorkflowProgressTracker } from './WorkflowProgressTracker';

// RED Phase: Test 8a - Tracks workflow creation progress
// These tests will fail initially because WorkflowProgressTracker doesn't exist yet

describe('WorkflowProgressTracker', () => {
  const mockOnStepComplete = jest.fn();
  const mockOnStepChange = jest.fn();
  const mockOnProgressUpdate = jest.fn();

  const sampleSteps = [
    {
      id: 'step-1',
      name: 'Basic Information',
      description: 'Enter workflow name and type',
      required: true,
      fields: ['name', 'type']
    },
    {
      id: 'step-2', 
      name: 'Service Configuration',
      description: 'Configure input and output services',
      required: true,
      fields: ['inputService', 'outputService']
    },
    {
      id: 'step-3',
      name: 'Triggers & Scheduling',
      description: 'Set up triggers and scheduling',
      required: false,
      fields: ['trigger', 'schedule']
    }
  ];

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('8a. Tracks workflow creation progress', () => {
    it('should initialize with step 1 of N', () => {
      // RED: This test will fail because WorkflowProgressTracker doesn't exist
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      expect(screen.getByText('Step 1 of 3')).toBeInTheDocument();
      expect(screen.getByText('Basic Information')).toBeInTheDocument();
      expect(screen.getByText('Enter workflow name and type')).toBeInTheDocument();
    });

    it('should track completion of each step', async () => {
      // RED: Test step completion tracking
      const user = userEvent.setup();
      const { rerender } = render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      // Mark step 1 as complete
      const completeButton = screen.getByText('Mark Complete');
      await user.click(completeButton);

      expect(mockOnStepComplete).toHaveBeenCalledWith('step-1', {
        name: 'Test Workflow',
        type: 'customer_support'
      });

      // Update to show step as completed
      rerender(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }}
          completedSteps={['step-1']}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      expect(screen.getByText('✓ Basic Information')).toBeInTheDocument();
      expect(screen.getByText('Step 2 of 3')).toBeInTheDocument();
    });

    it('should calculate overall progress percentage', () => {
      // RED: Test progress calculation
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          completedSteps={['step-1']}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const progressBar = screen.getByRole('progressbar');
      expect(progressBar).toHaveAttribute('aria-valuenow', '33'); // 1 of 3 steps completed
      expect(screen.getByText('33% Complete')).toBeInTheDocument();
    });

    it('should store step data independently', () => {
      // RED: Test independent step data storage
      const stepData = {
        'step-1': { name: 'Test Workflow', type: 'customer_support' },
        'step-2': { inputService: 'helpscout', outputService: 'slack' }
      };

      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          stepData={stepData}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      // Should show data from step 1 preserved
      expect(screen.getByDisplayValue('Test Workflow')).toBeInTheDocument();
      // Should show current step 2 data
      expect(screen.getByDisplayValue('helpscout')).toBeInTheDocument();
    });

    it('should update progress when steps are completed', async () => {
      // RED: Test progress update callbacks
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const completeButton = screen.getByText('Mark Complete');
      await user.click(completeButton);

      expect(mockOnProgressUpdate).toHaveBeenCalledWith({
        currentStep: 2,
        totalSteps: 3,
        completedSteps: ['step-1'],
        overallProgress: 33
      });
    });

    it('should handle step validation before marking complete', async () => {
      // RED: Test step validation
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: '', type: '' } // Invalid data
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const completeButton = screen.getByText('Mark Complete');
      await user.click(completeButton);

      // Should show validation errors
      expect(screen.getByText('Name is required')).toBeInTheDocument();
      expect(screen.getByText('Type is required')).toBeInTheDocument();
      
      // Should not call completion callback
      expect(mockOnStepComplete).not.toHaveBeenCalled();
    });
  });

  describe('8b. Allows navigation between steps', () => {
    it('should navigate to next step when current is valid', async () => {
      // RED: Test next step navigation
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const nextButton = screen.getByText('Next');
      await user.click(nextButton);

      expect(mockOnStepChange).toHaveBeenCalledWith(2);
    });

    it('should navigate to previous step', async () => {
      // RED: Test previous step navigation
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          completedSteps={['step-1']}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' },
            'step-2': { inputService: 'helpscout' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const prevButton = screen.getByText('Previous');
      await user.click(prevButton);

      expect(mockOnStepChange).toHaveBeenCalledWith(1);
    });

    it('should disable next on invalid step', () => {
      // RED: Test navigation validation
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: '', type: '' } // Invalid
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const nextButton = screen.getByText('Next');
      expect(nextButton).toBeDisabled();
    });

    it('should show step indicators', () => {
      // RED: Test step indicators
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          completedSteps={['step-1']}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      // Should show all step indicators
      expect(screen.getByLabelText('Step 1: Basic Information')).toBeInTheDocument();
      expect(screen.getByLabelText('Step 2: Service Configuration')).toBeInTheDocument();
      expect(screen.getByLabelText('Step 3: Triggers & Scheduling')).toBeInTheDocument();

      // Should indicate completion status
      const step1Indicator = screen.getByLabelText('Step 1: Basic Information');
      expect(step1Indicator).toHaveClass('completed');

      const step2Indicator = screen.getByLabelText('Step 2: Service Configuration');
      expect(step2Indicator).toHaveClass('current');
    });

    it('should allow clicking to navigate if step is accessible', async () => {
      // RED: Test click navigation
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          completedSteps={['step-1']}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      // Should be able to click on completed step
      const step1Indicator = screen.getByLabelText('Step 1: Basic Information');
      await user.click(step1Indicator);

      expect(mockOnStepChange).toHaveBeenCalledWith(1);
    });

    it('should not allow clicking to navigate to incomplete future steps', async () => {
      // RED: Test navigation restrictions
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      // Should not be able to click on future steps
      const step3Indicator = screen.getByLabelText('Step 3: Triggers & Scheduling');
      await user.click(step3Indicator);

      expect(mockOnStepChange).not.toHaveBeenCalledWith(3);
    });
  });

  describe('8c. Saves partial progress', () => {
    it('should save progress to localStorage', async () => {
      // RED: Test localStorage saving
      const user = userEvent.setup();
      const localStorageSpy = jest.spyOn(Storage.prototype, 'setItem');

      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
          saveToLocalStorage={true}
        />
      );

      const nameInput = screen.getByDisplayValue('Test Workflow');
      await user.type(nameInput, ' Updated');

      await waitFor(() => {
        expect(localStorageSpy).toHaveBeenCalledWith(
          'workflow-progress',
          expect.stringContaining('Test Workflow Updated')
        );
      });
    });

    it('should restore progress on reload', () => {
      // RED: Test progress restoration
      const savedProgress = {
        currentStep: 2,
        stepData: {
          'step-1': { name: 'Restored Workflow', type: 'customer_support' },
          'step-2': { inputService: 'helpscout' }
        },
        completedSteps: ['step-1']
      };

      jest.spyOn(Storage.prototype, 'getItem').mockReturnValue(JSON.stringify(savedProgress));

      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
          saveToLocalStorage={true}
        />
      );

      expect(screen.getByText('Step 2 of 3')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Restored Workflow')).toBeInTheDocument();
      expect(screen.getByText('✓ Basic Information')).toBeInTheDocument();
    });

    it('should save progress to backend via callback', async () => {
      // RED: Test backend saving
      const mockSaveToBackend = jest.fn();
      const user = userEvent.setup();

      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test Workflow' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
          onSaveProgress={mockSaveToBackend}
        />
      );

      const typeSelect = screen.getByLabelText('Type');
      await user.selectOptions(typeSelect, 'customer_support');

      await waitFor(() => {
        expect(mockSaveToBackend).toHaveBeenCalledWith({
          currentStep: 1,
          stepData: {
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }
        });
      });
    });

    it('should handle save failures gracefully', async () => {
      // RED: Test save error handling
      const mockSaveToBackend = jest.fn().mockRejectedValue(new Error('Save failed'));
      const user = userEvent.setup();

      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test Workflow' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
          onSaveProgress={mockSaveToBackend}
        />
      );

      const typeSelect = screen.getByLabelText('Type');
      await user.selectOptions(typeSelect, 'customer_support');

      await waitFor(() => {
        expect(screen.getByText('Auto-save failed. Your progress is saved locally.')).toBeInTheDocument();
      });
    });
  });

  describe('8d. Shows step indicators', () => {
    it('should display visual step progress', () => {
      // RED: Test visual progress
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          completedSteps={['step-1']}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const progressIndicator = screen.getByTestId('step-progress-indicator');
      expect(progressIndicator).toBeInTheDocument();

      // Should show all steps in order
      const stepNumbers = screen.getAllByTestId(/step-number-/);
      expect(stepNumbers).toHaveLength(3);
    });

    it('should highlight current step', () => {
      // RED: Test current step highlighting
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={2}
          completedSteps={['step-1']}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const currentStepIndicator = screen.getByTestId('step-number-2');
      expect(currentStepIndicator).toHaveClass('current');
      expect(currentStepIndicator).toHaveAttribute('aria-current', 'step');
    });

    it('should show completed steps with checkmarks', () => {
      // RED: Test completed step visualization
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={3}
          completedSteps={['step-1', 'step-2']}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const completedStep1 = screen.getByTestId('step-number-1');
      expect(completedStep1).toHaveClass('completed');
      expect(completedStep1).toHaveTextContent('✓');

      const completedStep2 = screen.getByTestId('step-number-2');
      expect(completedStep2).toHaveClass('completed');
      expect(completedStep2).toHaveTextContent('✓');
    });

    it('should show step titles and descriptions', () => {
      // RED: Test step information display
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      expect(screen.getByText('Basic Information')).toBeInTheDocument();
      expect(screen.getByText('Enter workflow name and type')).toBeInTheDocument();
    });
  });

  describe('8e. Handles step validation', () => {
    it('should validate required fields before next step', async () => {
      // RED: Test validation before navigation
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: '', type: '' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const nextButton = screen.getByText('Next');
      await user.click(nextButton);

      expect(screen.getByText('Please complete all required fields')).toBeInTheDocument();
      expect(mockOnStepChange).not.toHaveBeenCalled();
    });

    it('should show field-specific errors', async () => {
      // RED: Test field validation
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: '', type: '' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const nextButton = screen.getByText('Next');
      await user.click(nextButton);

      expect(screen.getByText('Name is required')).toBeInTheDocument();
      expect(screen.getByText('Type is required')).toBeInTheDocument();
    });

    it('should prevent navigation with invalid data', async () => {
      // RED: Test navigation blocking
      const user = userEvent.setup();
      render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test', type: '' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      const nextButton = screen.getByText('Next');
      expect(nextButton).toBeDisabled();
    });

    it('should clear errors when corrected', async () => {
      // RED: Test error clearing
      const user = userEvent.setup();
      const { rerender } = render(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: '', type: '' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      // Show validation errors
      const nextButton = screen.getByText('Next');
      await user.click(nextButton);
      expect(screen.getByText('Name is required')).toBeInTheDocument();

      // Fix the data
      rerender(
        <WorkflowProgressTracker
          steps={sampleSteps}
          currentStep={1}
          stepData={{
            'step-1': { name: 'Test Workflow', type: 'customer_support' }
          }}
          onStepComplete={mockOnStepComplete}
          onStepChange={mockOnStepChange}
          onProgressUpdate={mockOnProgressUpdate}
        />
      );

      expect(screen.queryByText('Name is required')).not.toBeInTheDocument();
      expect(nextButton).not.toBeDisabled();
    });
  });
});