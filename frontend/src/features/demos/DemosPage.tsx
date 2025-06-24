import React, { useState } from 'react';
import { Card, Button, Row, Col, Typography, Space, Tag, Modal, Input, message, Steps, Alert } from 'antd';
import {
  RocketOutlined,
  CustomerServiceOutlined,
  FileSearchOutlined,
  BulbOutlined,
  PlayCircleOutlined,
  CheckCircleOutlined,
  LoadingOutlined,
} from '@ant-design/icons';
import { workflowStore } from '../../stores/workflowStore';

const { Title, Text, Paragraph } = Typography;
const { TextArea } = Input;

interface DemoScenario {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  category: string;
  difficulty: 'Beginner' | 'Intermediate' | 'Advanced';
  estimatedTime: string;
  workflowName: string;
  sampleInput: any;
  expectedOutput: string;
  businessValue: string[];
}

const DemosPage: React.FC = () => {
  const { triggerWorkflow, isLoading } = workflowStore();
  const [selectedDemo, setSelectedDemo] = useState<DemoScenario | null>(null);
  const [modalVisible, setModalVisible] = useState(false);
  const [executionStep, setExecutionStep] = useState(0);
  const [demoResults, setDemoResults] = useState<any>(null);
  
  const demoScenarios: DemoScenario[] = [
    {
      id: 'customer-support',
      title: 'Intelligent Customer Support',
      description: 'Automate customer inquiry processing with AI-powered analysis, sentiment detection, and automated response generation.',
      icon: <CustomerServiceOutlined />,
      color: '#52c41a',
      category: 'Customer Service',
      difficulty: 'Beginner',
      estimatedTime: '2-3 minutes',
      workflowName: 'customer_care_workflow',
      sampleInput: {
        ticket_id: 'DEMO-001',
        customer_email: 'demo@example.com',
        subject: 'Unable to process payment',
        message: 'Hi, I\'m trying to update my payment method but keep getting an error. Can you help?',
        priority: 'medium',
        category: 'billing'
      },
      expectedOutput: 'Automated ticket analysis, sentiment assessment, and personalized response with resolution steps',
      businessValue: [
        'Reduce response time by 75%',
        'Improve customer satisfaction',
        'Scale support operations efficiently',
        'Consistent quality responses'
      ]
    },
    {
      id: 'research-documentation',
      title: 'Research & Documentation',
      description: 'Transform research queries into comprehensive documentation using AI agents, web search, and intelligent content synthesis.',
      icon: <FileSearchOutlined />,
      color: '#1890ff',
      category: 'Content Generation',
      difficulty: 'Intermediate',
      estimatedTime: '3-5 minutes',
      workflowName: 'research_to_documentation',
      sampleInput: {
        topic: 'AI workflow orchestration best practices',
        depth: 'comprehensive',
        format: 'markdown',
        include_examples: true,
        target_audience: 'developers'
      },
      expectedOutput: 'Structured documentation with research findings, code examples, and best practices',
      businessValue: [
        'Accelerate knowledge creation',
        'Ensure documentation consistency',
        'Reduce manual research time',
        'Improve knowledge sharing'
      ]
    },
    {
      id: 'knowledge-synthesis',
      title: 'Knowledge Base Integration',
      description: 'Query multiple knowledge sources, synthesize information, and generate intelligent responses using MCP protocol.',
      icon: <BulbOutlined />,
      color: '#722ed1',
      category: 'Knowledge Management',
      difficulty: 'Advanced',
      estimatedTime: '4-6 minutes',
      workflowName: 'knowledge_base_workflow',
      sampleInput: {
        query: 'How to implement event sourcing in microservices architecture?',
        sources: ['notion', 'documentation', 'code_repository'],
        response_format: 'detailed',
        include_code_examples: true
      },
      expectedOutput: 'Comprehensive answer with information from multiple sources, code examples, and implementation guidance',
      businessValue: [
        'Centralize knowledge access',
        'Improve decision making',
        'Reduce information silos',
        'Enable self-service learning'
      ]
    }
  ];
  
  const handleRunDemo = (demo: DemoScenario) => {
    setSelectedDemo(demo);
    setModalVisible(true);
    setExecutionStep(0);
    setDemoResults(null);
  };
  
  const executeDemo = async () => {
    if (!selectedDemo) return;
    
    try {
      setExecutionStep(1); // Starting execution
      
      const instanceId = await triggerWorkflow(
        selectedDemo.workflowName,
        selectedDemo.sampleInput
      );
      
      setExecutionStep(2); // Execution started
      message.success(`Demo workflow started! Instance ID: ${instanceId}`);
      
      // Simulate demo completion for showcase purposes
      setTimeout(() => {
        setExecutionStep(3); // Completed
        setDemoResults({
          instanceId,
          status: 'Completed',
          output: generateMockOutput(selectedDemo)
        });
      }, 3000);
      
    } catch (error: any) {
      message.error(error.message || 'Failed to start demo');
      setExecutionStep(0);
    }
  };
  
  const generateMockOutput = (demo: DemoScenario) => {
    switch (demo.id) {
      case 'customer-support':
        return {
          analysis: {
            sentiment: 'neutral',
            category: 'billing_support',
            priority: 'medium',
            urgency_score: 0.6
          },
          response: {
            greeting: 'Thank you for contacting our support team.',
            solution: 'I can help you update your payment method. Please try clearing your browser cache and cookies, then attempt the update again. If the issue persists, please check that your card details are correct and that your bank has not blocked the transaction.',
            next_steps: ['Clear browser cache', 'Verify card details', 'Contact bank if needed'],
            escalation: false
          },
          resolution_time: '2.3 seconds'
        };
      case 'research-documentation':
        return {
          title: 'AI Workflow Orchestration Best Practices',
          sections: [
            'Introduction to Workflow Orchestration',
            'Architecture Patterns',
            'Implementation Guidelines',
            'Performance Optimization',
            'Security Considerations'
          ],
          word_count: 2847,
          sources_consulted: 15,
          code_examples: 8
        };
      default:
        return {
          status: 'completed',
          processing_time: '4.7 seconds',
          sources_analyzed: 5,
          confidence_score: 0.92
        };
    }
  };
  
  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case 'Beginner': return 'green';
      case 'Intermediate': return 'orange';
      case 'Advanced': return 'red';
      default: return 'default';
    }
  };
  
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <Title level={2}>
          <RocketOutlined className="mr-3" />
          Live AI Workflow Demos
        </Title>
        <Paragraph className="text-lg text-gray-600 max-w-3xl mx-auto">
          Experience the power of our AI workflow orchestration platform through interactive demonstrations. 
          See real-world scenarios in action and understand the business impact.
        </Paragraph>
      </div>
      
      {/* Demo Scenarios Grid */}
      <Row gutter={[24, 24]}>
        {demoScenarios.map((demo) => (
          <Col xs={24} lg={8} key={demo.id}>
            <Card
              hoverable
              className="h-full"
              cover={
                <div 
                  className="h-32 flex items-center justify-center text-6xl"
                  style={{ backgroundColor: demo.color + '15', color: demo.color }}
                >
                  {demo.icon}
                </div>
              }
              actions={[
                <Button
                  type="primary"
                  icon={<PlayCircleOutlined />}
                  onClick={() => handleRunDemo(demo)}
                  block
                >
                  Run Demo
                </Button>
              ]}
            >
              <Card.Meta
                title={
                  <div className="flex justify-between items-center">
                    <span>{demo.title}</span>
                    <Tag color={getDifficultyColor(demo.difficulty)}>
                      {demo.difficulty}
                    </Tag>
                  </div>
                }
                description={
                  <div className="space-y-3">
                    <Paragraph className="text-sm">{demo.description}</Paragraph>
                    
                    <div className="flex justify-between text-xs text-gray-500">
                      <span>⏱️ {demo.estimatedTime}</span>
                      <span>📁 {demo.category}</span>
                    </div>
                    
                    <div>
                      <Text strong className="block mb-2">Business Value:</Text>
                      <ul className="text-xs space-y-1">
                        {demo.businessValue.slice(0, 2).map((value, index) => (
                          <li key={index} className="text-gray-600">✓ {value}</li>
                        ))}
                      </ul>
                    </div>
                  </div>
                }
              />
            </Card>
          </Col>
        ))}
      </Row>
      
      {/* Platform Capabilities Overview */}
      <Card title="Platform Capabilities Showcase">
        <Row gutter={[16, 16]}>
          <Col xs={24} md={12} lg={6}>
            <div className="text-center p-4">
              <div className="text-3xl mb-2">🤖</div>
              <Title level={5}>AI Integration</Title>
              <Text type="secondary">OpenAI, Anthropic, AWS Bedrock</Text>
            </div>
          </Col>
          <Col xs={24} md={12} lg={6}>
            <div className="text-center p-4">
              <div className="text-3xl mb-2">🔗</div>
              <Title level={5}>MCP Protocol</Title>
              <Text type="secondary">External tool connectivity</Text>
            </div>
          </Col>
          <Col xs={24} md={12} lg={6}>
            <div className="text-center p-4">
              <div className="text-3xl mb-2">⚡</div>
              <Title level={5}>Event Sourcing</Title>
              <Text type="secondary">Complete audit trail</Text>
            </div>
          </Col>
          <Col xs={24} md={12} lg={6}>
            <div className="text-center p-4">
              <div className="text-3xl mb-2">📊</div>
              <Title level={5}>Real-time Monitoring</Title>
              <Text type="secondary">Performance insights</Text>
            </div>
          </Col>
        </Row>
      </Card>
      
      {/* Demo Execution Modal */}
      <Modal
        title={selectedDemo?.title}
        open={modalVisible}
        onCancel={() => setModalVisible(false)}
        width={800}
        footer={
          <Space>
            <Button onClick={() => setModalVisible(false)}>Close</Button>
            {executionStep === 0 && (
              <Button
                type="primary"
                icon={<PlayCircleOutlined />}
                loading={isLoading}
                onClick={executeDemo}
              >
                Execute Demo
              </Button>
            )}
          </Space>
        }
      >
        {selectedDemo && (
          <div className="space-y-6">
            {/* Demo Info */}
            <Alert
              message="Demo Scenario"
              description={selectedDemo.description}
              type="info"
              showIcon
            />
            
            {/* Execution Steps */}
            <Steps
              current={executionStep}
              items={[
                {
                  title: 'Ready',
                  description: 'Demo configuration prepared',
                  icon: <CheckCircleOutlined />,
                },
                {
                  title: 'Executing',
                  description: 'AI workflow processing',
                  icon: executionStep === 1 ? <LoadingOutlined /> : undefined,
                },
                {
                  title: 'Processing',
                  description: 'Generating results',
                  icon: executionStep === 2 ? <LoadingOutlined /> : undefined,
                },
                {
                  title: 'Complete',
                  description: 'Results ready',
                  icon: executionStep === 3 ? <CheckCircleOutlined /> : undefined,
                },
              ]}
            />
            
            {/* Input Data Preview */}
            <div>
              <Title level={5}>Sample Input Data:</Title>
              <TextArea
                value={JSON.stringify(selectedDemo.sampleInput, null, 2)}
                rows={6}
                readOnly
              />
            </div>
            
            {/* Results */}
            {demoResults && (
              <div>
                <Title level={5}>Demo Results:</Title>
                <Card className="bg-green-50 border-green-200">
                  <Space direction="vertical" className="w-full">
                    <div>
                      <Text strong>Status:</Text> 
                      <Tag color="success" className="ml-2">{demoResults.status}</Tag>
                    </div>
                    <div>
                      <Text strong>Instance ID:</Text> 
                      <Text code className="ml-2">{demoResults.instanceId}</Text>
                    </div>
                    <div>
                      <Text strong>Output Preview:</Text>
                      <TextArea
                        value={JSON.stringify(demoResults.output, null, 2)}
                        rows={8}
                        readOnly
                        className="mt-2"
                      />
                    </div>
                  </Space>
                </Card>
              </div>
            )}
            
            {/* Expected Output */}
            <div>
              <Title level={5}>Expected Output:</Title>
              <Text>{selectedDemo.expectedOutput}</Text>
            </div>
            
            {/* Business Value */}
            <div>
              <Title level={5}>Business Value:</Title>
              <ul className="space-y-1">
                {selectedDemo.businessValue.map((value, index) => (
                  <li key={index} className="flex items-center">
                    <CheckCircleOutlined className="text-green-500 mr-2" />
                    {value}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};

export default DemosPage;