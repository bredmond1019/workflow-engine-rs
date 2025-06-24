import React, { useEffect, useState } from 'react';
import { Row, Col, Card, Statistic, Progress, Space, Typography, Spin, Tag, Timeline } from 'antd';
import {
  CheckCircleOutlined,
  SyncOutlined,
  CloseCircleOutlined,
  RocketOutlined,
  ApiOutlined,
  CloudServerOutlined,
  DollarOutlined,
} from '@ant-design/icons';
import { Line, Pie } from '@ant-design/charts';
import { workflowStore } from '../../stores/workflowStore';
import { WorkflowStatus } from '../../types';

const { Title, Text } = Typography;

const DashboardPage: React.FC = () => {
  const { instances, fetchAllInstances } = workflowStore();
  const [loading, setLoading] = useState(true);
  const [stats, setStats] = useState({
    total: 0,
    completed: 0,
    running: 0,
    failed: 0,
    successRate: 0,
  });
  
  useEffect(() => {
    loadDashboardData();
  }, []);
  
  useEffect(() => {
    calculateStats();
  }, [instances]);
  
  const loadDashboardData = async () => {
    setLoading(true);
    try {
      await fetchAllInstances();
    } finally {
      setLoading(false);
    }
  };
  
  const calculateStats = () => {
    const instanceArray = Array.from(instances.values());
    const total = instanceArray.length;
    const completed = instanceArray.filter(i => i.status === WorkflowStatus.Completed).length;
    const running = instanceArray.filter(i => i.status === WorkflowStatus.Running).length;
    const failed = instanceArray.filter(i => i.status === WorkflowStatus.Failed).length;
    const successRate = total > 0 ? (completed / total) * 100 : 0;
    
    setStats({ total, completed, running, failed, successRate });
  };
  
  // Mock data for charts
  const executionTrendData = [
    { date: '2024-01-01', executions: 45 },
    { date: '2024-01-02', executions: 52 },
    { date: '2024-01-03', executions: 61 },
    { date: '2024-01-04', executions: 58 },
    { date: '2024-01-05', executions: 73 },
    { date: '2024-01-06', executions: 68 },
    { date: '2024-01-07', executions: 82 },
  ];
  
  const workflowDistributionData = [
    { type: 'Customer Support', value: 35 },
    { type: 'Data Analysis', value: 25 },
    { type: 'Content Generation', value: 20 },
    { type: 'Research', value: 15 },
    { type: 'Other', value: 5 },
  ];
  
  const lineConfig = {
    data: executionTrendData,
    xField: 'date',
    yField: 'executions',
    smooth: true,
    point: { size: 4 },
    label: {},
    xAxis: {
      label: {
        autoRotate: false,
      },
    },
  };
  
  const pieConfig = {
    data: workflowDistributionData,
    angleField: 'value',
    colorField: 'type',
    radius: 0.8,
    label: {
      type: 'outer',
      content: '{name} {percentage}',
    },
    interactions: [{ type: 'element-active' }],
  };
  
  if (loading) {
    return (
      <div className="flex justify-center items-center h-96">
        <Spin size="large" tip="Loading dashboard..." />
      </div>
    );
  }
  
  return (
    <div className="space-y-6">
      <div>
        <Title level={2}>Dashboard Overview</Title>
        <Text type="secondary">Monitor your AI workflow orchestration system</Text>
      </div>
      
      {/* Key Metrics */}
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} lg={6}>
          <Card hoverable>
            <Statistic
              title="Total Workflows"
              value={stats.total}
              prefix={<RocketOutlined />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card hoverable>
            <Statistic
              title="Completed"
              value={stats.completed}
              prefix={<CheckCircleOutlined />}
              valueStyle={{ color: '#52c41a' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card hoverable>
            <Statistic
              title="Running"
              value={stats.running}
              prefix={<SyncOutlined spin />}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card hoverable>
            <Statistic
              title="Failed"
              value={stats.failed}
              prefix={<CloseCircleOutlined />}
              valueStyle={{ color: '#ff4d4f' }}
            />
          </Card>
        </Col>
      </Row>
      
      {/* Performance Metrics */}
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={8}>
          <Card
            title="Success Rate"
            extra={<Tag color="success">Live</Tag>}
          >
            <Progress
              type="dashboard"
              percent={Math.round(stats.successRate)}
              strokeColor={{
                '0%': '#108ee9',
                '100%': '#87d068',
              }}
            />
            <div className="text-center mt-4">
              <Text type="secondary">
                {stats.completed} successful out of {stats.total} total
              </Text>
            </div>
          </Card>
        </Col>
        
        <Col xs={24} lg={8}>
          <Card
            title="System Health"
            extra={<Tag color="success">Healthy</Tag>}
          >
            <Space direction="vertical" className="w-full">
              <div className="flex justify-between items-center">
                <Space>
                  <ApiOutlined />
                  <Text>API Server</Text>
                </Space>
                <Tag color="success">Online</Tag>
              </div>
              <div className="flex justify-between items-center">
                <Space>
                  <CloudServerOutlined />
                  <Text>MCP Servers</Text>
                </Space>
                <Tag color="success">3/3 Connected</Tag>
              </div>
              <div className="flex justify-between items-center">
                <Space>
                  <DollarOutlined />
                  <Text>AI Credits</Text>
                </Space>
                <Tag color="warning">$423.50 Used</Tag>
              </div>
            </Space>
          </Card>
        </Col>
        
        <Col xs={24} lg={8}>
          <Card title="Recent Activity">
            <Timeline
              items={[
                {
                  color: 'green',
                  children: 'Customer support workflow completed',
                },
                {
                  color: 'blue',
                  children: 'Research workflow started',
                },
                {
                  color: 'red',
                  children: 'Data analysis workflow failed',
                },
                {
                  color: 'green',
                  children: 'Content generation completed',
                },
              ]}
            />
          </Card>
        </Col>
      </Row>
      
      {/* Charts */}
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card title="Execution Trend (Last 7 Days)">
            <Line {...lineConfig} />
          </Card>
        </Col>
        
        <Col xs={24} lg={12}>
          <Card title="Workflow Distribution">
            <Pie {...pieConfig} />
          </Card>
        </Col>
      </Row>
      
      {/* Key Features Showcase */}
      <Card title="System Capabilities">
        <Row gutter={[16, 16]}>
          <Col xs={24} sm={12} md={6}>
            <Card className="text-center hover:shadow-lg transition-shadow">
              <RocketOutlined className="text-4xl text-blue-500 mb-4" />
              <Title level={5}>AI Integration</Title>
              <Text type="secondary">OpenAI, Anthropic, AWS Bedrock</Text>
            </Card>
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Card className="text-center hover:shadow-lg transition-shadow">
              <ApiOutlined className="text-4xl text-green-500 mb-4" />
              <Title level={5}>MCP Protocol</Title>
              <Text type="secondary">External tool integration</Text>
            </Card>
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Card className="text-center hover:shadow-lg transition-shadow">
              <SyncOutlined className="text-4xl text-purple-500 mb-4" />
              <Title level={5}>Event Sourcing</Title>
              <Text type="secondary">Complete audit trail</Text>
            </Card>
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Card className="text-center hover:shadow-lg transition-shadow">
              <CloudServerOutlined className="text-4xl text-orange-500 mb-4" />
              <Title level={5}>Microservices</Title>
              <Text type="secondary">Scalable architecture</Text>
            </Card>
          </Col>
        </Row>
      </Card>
    </div>
  );
};

export default DashboardPage;