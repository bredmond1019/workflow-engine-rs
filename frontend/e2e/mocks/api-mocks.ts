/**
 * Mock API responses for e2e tests
 * 
 * NOTE: These mocks replace actual API calls during testing.
 * In production, these would be real calls to:
 * - LLM APIs (OpenAI, Anthropic, AWS Bedrock)
 * - MCP Servers (HelpScout, Notion, Slack)
 * - Backend workflow orchestration
 */

// ==========================================
// AUTH MOCKS
// ==========================================
export const mockAuthToken = {
  access_token: 'mock-jwt-token-for-testing',
  token_type: 'Bearer',
  expires_in: 86400
};

export const mockUser = {
  sub: 'test_user',
  role: 'admin',
  exp: Math.floor(Date.now() / 1000) + 86400,
  iat: Math.floor(Date.now() / 1000)
};

// ==========================================
// WORKFLOW MOCKS
// ==========================================
export const mockAvailableWorkflows = {
  workflows: [
    'customer_care_workflow',
    'research_to_documentation',
    'knowledge_base_workflow'
  ]
};

export const mockWorkflowTriggerResponse = {
  instance_id: 'test-workflow-123',
  workflow_name: 'customer_care_workflow',
  status: 'Created',
  created_at: new Date().toISOString()
};

// ==========================================
// LLM API MOCKS
// ==========================================

/**
 * Mock OpenAI API Response
 * NOTE: In production, this would be a real call to OpenAI API
 * Requires: OPENAI_API_KEY environment variable
 */
export const mockOpenAIResponse = {
  id: 'chatcmpl-mock-123',
  object: 'chat.completion',
  created: Date.now(),
  model: 'gpt-4',
  choices: [{
    index: 0,
    message: {
      role: 'assistant',
      content: 'Based on the customer\'s inquiry about their delayed order #12345, I can see that the package was shipped on March 15th and is currently in transit. The tracking shows it\'s delayed due to weather conditions in the Midwest. Expected delivery is now March 22nd. I recommend offering the customer expedited shipping on their next order as compensation.'
    },
    finish_reason: 'stop'
  }],
  usage: {
    prompt_tokens: 150,
    completion_tokens: 75,
    total_tokens: 225
  }
};

/**
 * Mock Anthropic Claude API Response
 * NOTE: In production, this would be a real call to Anthropic API
 * Requires: ANTHROPIC_API_KEY environment variable
 */
export const mockAnthropicResponse = {
  id: 'msg_mock_456',
  type: 'message',
  role: 'assistant',
  content: [{
    type: 'text',
    text: 'I\'ve analyzed the customer support ticket. The customer is experiencing frustration due to a delayed shipment. Key points:\n1. Order placed 10 days ago\n2. Expected delivery was 3 days ago\n3. No tracking updates for 48 hours\n\nRecommended actions:\n- Immediately check with shipping carrier\n- Offer full refund or replacement\n- Provide discount code for future purchase\n- Escalate to supervisor if customer remains unsatisfied'
  }],
  model: 'claude-3-opus-20240229',
  stop_reason: 'end_turn',
  usage: {
    input_tokens: 200,
    output_tokens: 120
  }
};

/**
 * Mock AWS Bedrock Response
 * NOTE: In production, this would be a real call to AWS Bedrock
 * Requires: AWS credentials and Bedrock model access
 */
export const mockBedrockResponse = {
  completion: 'After reviewing the documentation requirements, I recommend structuring the technical guide with the following sections:\n\n1. Introduction and Overview\n2. System Architecture\n3. API Reference\n4. Code Examples\n5. Troubleshooting Guide\n6. FAQ Section\n\nEach section should include practical examples and clear explanations suitable for developers.',
  stop_reason: 'stop',
  metrics: {
    inputTokens: 100,
    outputTokens: 80
  }
};

// ==========================================
// MCP SERVER MOCKS
// ==========================================

/**
 * Mock HelpScout MCP Server Response
 * NOTE: In production, this would be a real call to HelpScout MCP server at port 8001
 * The MCP server would authenticate with HelpScout API using HELPSCOUT_API_KEY
 */
export const mockHelpScoutMCPResponse = {
  tool: 'get_ticket_details',
  result: {
    ticket_id: 'HS-12345',
    customer: {
      name: 'John Doe',
      email: 'john.doe@example.com',
      previous_tickets: 3
    },
    subject: 'Order Delayed - Need Update',
    priority: 'high',
    status: 'pending',
    created_at: '2024-03-15T10:30:00Z',
    messages: [
      {
        from: 'customer',
        content: 'My order #98765 was supposed to arrive 3 days ago. Can you please check?',
        timestamp: '2024-03-15T10:30:00Z'
      }
    ],
    order_info: {
      order_id: '98765',
      status: 'in_transit',
      tracking_number: 'TRACK123456',
      estimated_delivery: '2024-03-22'
    }
  }
};

/**
 * Mock Notion MCP Server Response
 * NOTE: In production, this would be a real call to Notion MCP server at port 8002
 * The MCP server would authenticate with Notion API using NOTION_API_KEY
 */
export const mockNotionMCPResponse = {
  tool: 'search_knowledge_base',
  result: {
    pages: [
      {
        id: 'notion-page-123',
        title: 'Shipping Delay Procedures',
        content: 'When handling shipping delays:\n1. Check tracking information\n2. Contact carrier if no updates for 48h\n3. Offer compensation based on delay duration\n4. Escalate to supervisor for delays > 7 days',
        last_updated: '2024-03-01T09:00:00Z',
        relevance_score: 0.95
      },
      {
        id: 'notion-page-456',
        title: 'Customer Compensation Guidelines',
        content: 'Compensation tiers:\n- 3-5 days delay: 10% discount\n- 6-10 days delay: 20% discount or free shipping\n- >10 days: Full refund option + 25% discount',
        last_updated: '2024-02-15T14:30:00Z',
        relevance_score: 0.87
      }
    ]
  }
};

/**
 * Mock Slack MCP Server Response
 * NOTE: In production, this would be a real call to Slack MCP server at port 8003
 * The MCP server would authenticate with Slack API using SLACK_BOT_TOKEN
 */
export const mockSlackMCPResponse = {
  tool: 'send_message',
  result: {
    ok: true,
    channel: 'C123456789',
    ts: '1234567890.123456',
    message: {
      text: '🚨 High Priority Support Ticket: Customer experiencing 7+ day shipping delay on order #98765. Escalation may be needed.',
      channel: 'support-escalations',
      user: 'workflow-bot'
    }
  }
};

// ==========================================
// WORKFLOW STATUS PROGRESSION MOCKS
// ==========================================

/**
 * Mock workflow status progression
 * Simulates the real-time updates that would come from the backend
 */
export const mockWorkflowStatusProgression = [
  // Initial state
  {
    instance_id: 'test-workflow-123',
    workflow_name: 'customer_care_workflow',
    status: 'Running',
    created_at: new Date().toISOString(),
    started_at: new Date().toISOString(),
    inputs: {
      ticket_id: 'HS-12345',
      priority: 'high'
    },
    progress: {
      total_steps: 5,
      completed_steps: 0,
      failed_steps: 0,
      running_steps: 1,
      percentage: 0
    },
    steps: {
      'fetch_ticket_details': {
        status: 'Running',
        started_at: new Date().toISOString(),
        attempt: 1
      }
    }
  },
  // After HelpScout MCP call
  {
    progress: {
      total_steps: 5,
      completed_steps: 1,
      failed_steps: 0,
      running_steps: 1,
      percentage: 20
    },
    steps: {
      'fetch_ticket_details': {
        status: 'Completed',
        output: mockHelpScoutMCPResponse.result,
        completed_at: new Date().toISOString(),
        attempt: 1
      },
      'analyze_with_ai': {
        status: 'Running',
        started_at: new Date().toISOString(),
        attempt: 1
      }
    }
  },
  // After AI analysis (OpenAI/Anthropic call)
  {
    progress: {
      total_steps: 5,
      completed_steps: 2,
      failed_steps: 0,
      running_steps: 1,
      percentage: 40
    },
    steps: {
      'analyze_with_ai': {
        status: 'Completed',
        output: {
          analysis: mockOpenAIResponse.choices[0].message.content,
          sentiment: 'frustrated',
          urgency: 'high',
          recommended_actions: ['check_tracking', 'offer_compensation', 'escalate_if_needed']
        },
        completed_at: new Date().toISOString(),
        attempt: 1
      },
      'search_knowledge_base': {
        status: 'Running',
        started_at: new Date().toISOString(),
        attempt: 1
      }
    }
  },
  // After Notion MCP call
  {
    progress: {
      total_steps: 5,
      completed_steps: 3,
      failed_steps: 0,
      running_steps: 1,
      percentage: 60
    },
    steps: {
      'search_knowledge_base': {
        status: 'Completed',
        output: mockNotionMCPResponse.result,
        completed_at: new Date().toISOString(),
        attempt: 1
      },
      'generate_response': {
        status: 'Running',
        started_at: new Date().toISOString(),
        attempt: 1
      }
    }
  },
  // After response generation (Anthropic call)
  {
    progress: {
      total_steps: 5,
      completed_steps: 4,
      failed_steps: 0,
      running_steps: 1,
      percentage: 80
    },
    steps: {
      'generate_response': {
        status: 'Completed',
        output: {
          response_text: 'Dear John, I sincerely apologize for the delay with your order #98765. I\'ve checked the tracking, and your package is currently in transit but delayed due to weather conditions. New estimated delivery is March 22nd. As compensation for this inconvenience, I\'m applying a 20% discount to your account for your next purchase. I\'ll personally monitor your shipment and update you daily.',
          tone: 'empathetic',
          includes_compensation: true
        },
        completed_at: new Date().toISOString(),
        attempt: 1
      },
      'notify_team': {
        status: 'Running',
        started_at: new Date().toISOString(),
        attempt: 1
      }
    }
  },
  // Final state after Slack notification
  {
    status: 'Completed',
    completed_at: new Date().toISOString(),
    progress: {
      total_steps: 5,
      completed_steps: 5,
      failed_steps: 0,
      running_steps: 0,
      percentage: 100
    },
    steps: {
      'notify_team': {
        status: 'Completed',
        output: mockSlackMCPResponse.result,
        completed_at: new Date().toISOString(),
        attempt: 1
      }
    },
    outputs: {
      ticket_resolved: true,
      customer_response: 'Generated and ready to send',
      compensation_offered: '20% discount',
      team_notified: true,
      total_processing_time: '45 seconds'
    }
  }
];

// ==========================================
// ERROR SCENARIO MOCKS
// ==========================================

/**
 * Mock API error responses for testing error handling
 */
export const mockAPIErrors = {
  // LLM API rate limit error
  openAIRateLimit: {
    error: {
      message: 'Rate limit exceeded for requests',
      type: 'rate_limit_error',
      code: 'rate_limit_exceeded'
    }
  },
  
  // MCP server connection error
  mcpConnectionError: {
    error: 'Failed to connect to MCP server',
    details: 'Connection refused on port 8001'
  },
  
  // Workflow execution error
  workflowExecutionError: {
    instance_id: 'test-workflow-error-456',
    status: 'Failed',
    error: {
      message: 'Step execution failed',
      code: 'STEP_EXECUTION_ERROR',
      step_id: 'analyze_with_ai',
      details: {
        reason: 'LLM API key invalid or expired'
      }
    }
  }
};