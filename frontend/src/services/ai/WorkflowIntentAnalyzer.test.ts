import { WorkflowIntentAnalyzer, WorkflowIntent, IntentType, WorkflowType, WorkflowParameter } from './WorkflowIntentAnalyzer';

describe('WorkflowIntentAnalyzer', () => {
  let analyzer: WorkflowIntentAnalyzer;

  beforeEach(() => {
    analyzer = new WorkflowIntentAnalyzer();
  });

  describe('Test 4a: Detects "create workflow" intent', () => {
    it('should detect explicit create workflow intent', async () => {
      const message = 'I want to create a new workflow';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.confidence).toBeGreaterThan(0.8);
    });

    it('should detect implicit create workflow intent', async () => {
      const message = 'Can you help me set up an automated process for handling customer support tickets?';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.confidence).toBeGreaterThan(0.7);
    });

    it('should detect various create workflow phrasings', async () => {
      const createPhrases = [
        'Build a workflow for me',
        'I need to automate something',
        'Set up a new automation',
        'Design a workflow that handles',
        'Make a process for',
        'Generate a workflow to',
        'Construct an automated system'
      ];

      for (const phrase of createPhrases) {
        const intent = await analyzer.analyzeIntent(phrase);
        expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
        expect(intent.confidence).toBeGreaterThan(0.7);
      }
    });

    it('should not detect create intent when asking about existing workflows', async () => {
      const message = 'Show me my existing workflows';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).not.toBe(IntentType.CREATE_WORKFLOW);
    });
  });

  describe('Test 4b: Identifies workflow type from description', () => {
    it('should identify customer support workflow type', async () => {
      const message = 'Create a workflow to handle customer support tickets from HelpScout';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.workflowType).toBe(WorkflowType.CUSTOMER_SUPPORT);
      expect(intent.suggestedTemplate).toContain('support');
    });

    it('should identify knowledge base workflow type', async () => {
      const message = 'I want to build a workflow that syncs documentation from Notion to our knowledge base';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.workflowType).toBe(WorkflowType.KNOWLEDGE_BASE);
      expect(intent.suggestedTemplate).toContain('knowledge');
    });

    it('should identify data processing workflow type', async () => {
      const message = 'Create an automation to process CSV files and generate reports';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.workflowType).toBe(WorkflowType.DATA_PROCESSING);
    });

    it('should identify communication workflow type', async () => {
      const message = 'Set up a workflow to send Slack notifications when certain events occur';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.workflowType).toBe(WorkflowType.COMMUNICATION);
      expect(intent.detectedServices).toContain('slack');
    });

    it('should identify AI/ML workflow type', async () => {
      const message = 'Build a workflow that uses GPT-4 to analyze customer feedback';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.workflowType).toBe(WorkflowType.AI_ML);
      expect(intent.detectedServices).toContain('openai');
    });

    it('should handle unknown workflow types', async () => {
      const message = 'Create a workflow for something completely new';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.workflowType).toBe(WorkflowType.GENERIC);
    });
  });

  describe('Test 4c: Extracts key parameters from natural language', () => {
    it('should extract trigger parameters', async () => {
      const message = 'Create a workflow that runs every Monday at 9 AM';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.parameters).toContainEqual({
        name: 'trigger',
        value: 'schedule',
        metadata: {
          frequency: 'weekly',
          day: 'monday',
          time: '09:00'
        }
      });
    });

    it('should extract service parameters', async () => {
      const message = 'Build a workflow that pulls data from HelpScout and posts to Slack channel #support';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.parameters).toContainEqual({
        name: 'source_service',
        value: 'helpscout',
        metadata: {}
      });
      expect(intent.parameters).toContainEqual({
        name: 'destination_service',
        value: 'slack',
        metadata: {
          channel: '#support'
        }
      });
    });

    it('should extract condition parameters', async () => {
      const message = 'Create a workflow that only processes high priority tickets';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.parameters).toContainEqual({
        name: 'condition',
        value: 'priority',
        metadata: {
          operator: 'equals',
          value: 'high'
        }
      });
    });

    it('should extract multiple parameters from complex descriptions', async () => {
      const message = 'I need a workflow that monitors HelpScout for urgent customer tickets, ' +
                     'uses AI to draft responses, and sends them to Slack for review before posting back';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.parameters.length).toBeGreaterThan(3);
      expect(intent.parameters.map(p => p.name)).toContain('source_service');
      expect(intent.parameters.map(p => p.name)).toContain('ai_processing');
      expect(intent.parameters.map(p => p.name)).toContain('review_channel');
    });

    it('should extract data transformation parameters', async () => {
      const message = 'Create a workflow that converts markdown files to PDF format';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.parameters).toContainEqual({
        name: 'transformation',
        value: 'format_conversion',
        metadata: {
          from: 'markdown',
          to: 'pdf'
        }
      });
    });
  });

  describe('Test 4d: Handles ambiguous requests with clarification', () => {
    it('should identify ambiguous workflow descriptions', async () => {
      const message = 'I want to create something';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.confidence).toBeLessThan(0.5);
      expect(intent.needsClarification).toBe(true);
      expect(intent.clarificationQuestions).toContain('What type of workflow would you like to create?');
    });

    it('should ask for clarification on missing trigger information', async () => {
      const message = 'Create a workflow for processing documents';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.needsClarification).toBe(true);
      expect(intent.clarificationQuestions).toContainEqual(
        expect.stringContaining('trigger')
      );
    });

    it('should ask for service clarification when multiple options exist', async () => {
      const message = 'Build a workflow to send messages';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.needsClarification).toBe(true);
      expect(intent.clarificationQuestions).toContainEqual(
        expect.stringContaining('which messaging service')
      );
      expect(intent.suggestedOptions).toContain('slack');
      expect(intent.suggestedOptions).toContain('email');
    });

    it('should provide suggestions for incomplete workflow descriptions', async () => {
      const message = 'I need automation for customer data';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.needsClarification).toBe(true);
      expect(intent.suggestions).toContainEqual({
        template: 'customer-data-sync',
        description: 'Sync customer data between systems'
      });
      expect(intent.suggestions).toContainEqual({
        template: 'customer-data-analysis',
        description: 'Analyze customer data for insights'
      });
    });

    it('should handle completely vague requests', async () => {
      const message = 'help me automate';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
      expect(intent.confidence).toBeLessThan(0.3);
      expect(intent.needsClarification).toBe(true);
      expect(intent.clarificationQuestions.length).toBeGreaterThan(2);
    });
  });

  describe('Test 4e: Recognizes workflow modification requests', () => {
    it('should detect workflow modification intent', async () => {
      const message = 'Modify my customer support workflow to include AI responses';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.MODIFY_WORKFLOW);
      expect(intent.targetWorkflow).toContain('customer support');
      expect(intent.modifications).toContainEqual({
        action: 'add',
        component: 'ai_processing',
        details: 'AI responses'
      });
    });

    it('should detect workflow update requests', async () => {
      const message = 'Update the schedule for my daily report workflow to run at 6 PM instead';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.MODIFY_WORKFLOW);
      expect(intent.targetWorkflow).toContain('daily report');
      expect(intent.modifications).toContainEqual({
        action: 'update',
        component: 'schedule',
        details: {
          time: '18:00'
        }
      });
    });

    it('should detect workflow deletion requests', async () => {
      const message = 'Delete the old backup workflow';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.DELETE_WORKFLOW);
      expect(intent.targetWorkflow).toContain('backup');
    });

    it('should detect adding steps to existing workflows', async () => {
      const message = 'Add a Slack notification step to my data processing workflow';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.MODIFY_WORKFLOW);
      expect(intent.targetWorkflow).toContain('data processing');
      expect(intent.modifications).toContainEqual({
        action: 'add',
        component: 'step',
        details: {
          type: 'notification',
          service: 'slack'
        }
      });
    });

    it('should detect removing steps from workflows', async () => {
      const message = 'Remove the email notification from my ticket workflow';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.MODIFY_WORKFLOW);
      expect(intent.targetWorkflow).toContain('ticket');
      expect(intent.modifications).toContainEqual({
        action: 'remove',
        component: 'notification',
        details: {
          type: 'email'
        }
      });
    });

    it('should detect workflow cloning requests', async () => {
      const message = 'Duplicate my sales workflow but change it for marketing';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.CLONE_WORKFLOW);
      expect(intent.sourceWorkflow).toContain('sales');
      expect(intent.modifications).toContainEqual({
        name: expect.stringContaining('marketing'),
        purpose: 'marketing'
      });
    });

    it('should handle ambiguous modification requests', async () => {
      const message = 'Change my workflow';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.MODIFY_WORKFLOW);
      expect(intent.needsClarification).toBe(true);
      expect(intent.clarificationQuestions).toContainEqual(
        expect.stringContaining('which workflow')
      );
      expect(intent.clarificationQuestions).toContainEqual(
        expect.stringContaining('what changes')
      );
    });
  });

  describe('Edge cases and error handling', () => {
    it('should handle empty messages', async () => {
      const intent = await analyzer.analyzeIntent('');
      
      expect(intent.type).toBe(IntentType.UNKNOWN);
      expect(intent.confidence).toBe(0);
    });

    it('should handle non-workflow related messages', async () => {
      const message = 'What is the weather today?';
      const intent = await analyzer.analyzeIntent(message);

      expect(intent.type).toBe(IntentType.UNKNOWN);
      expect(intent.confidence).toBeLessThan(0.2);
    });

    it('should handle very long messages gracefully', async () => {
      const longMessage = 'Create a workflow that ' + 'does many things '.repeat(100);
      const intent = await analyzer.analyzeIntent(longMessage);

      expect(intent).toBeDefined();
      expect(intent.type).toBe(IntentType.CREATE_WORKFLOW);
    });

    it('should preserve context from previous analyses', async () => {
      // First message establishes context
      await analyzer.analyzeIntent('I want to create a customer support workflow');
      
      // Follow-up message uses context
      const intent = await analyzer.analyzeIntent('Make it run every hour');
      
      expect(intent.type).toBe(IntentType.MODIFY_WORKFLOW);
      expect(intent.contextualReference).toBe(true);
      expect(intent.parameters).toContainEqual({
        name: 'trigger',
        value: 'schedule',
        metadata: {
          frequency: 'hourly'
        }
      });
    });
  });
});