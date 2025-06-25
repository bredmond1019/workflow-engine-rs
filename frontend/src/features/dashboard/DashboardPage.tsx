import React, { useEffect, useState } from 'react';
import { Row, Col, Card, Statistic, Progress, Space, Typography, Spin, Tag, Timeline, Button } from 'antd';
import {
  CheckCircleOutlined,
  SyncOutlined,
  CloseCircleOutlined,
  RocketOutlined,
  ApiOutlined,
  CloudServerOutlined,
  PlusOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { workflowStore } from '../../stores/workflowStore';
import { WorkflowStatus } from '../../types';

const { Title, Text } = Typography;

const DashboardPage: React.FC = () => {
  const navigate = useNavigate();
  const { instances, fetchAllInstances, startPolling, stopPolling } = workflowStore();
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
    startPolling();
    
    return () => {
      stopPolling();
    };
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
  
  const getRecentActivity = () => {
    const recentInstances = Array.from(instances.values())
      .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      .slice(0, 5);
      
    return recentInstances.map((instance) => ({
      color: instance.status === WorkflowStatus.Completed ? 'green' :
             instance.status === WorkflowStatus.Running ? 'blue' :
             instance.status === WorkflowStatus.Failed ? 'red' : 'gray',
      children: (
        <div>
          <Text strong>{instance.workflow_name.replace(/_/g, ' ')}</Text>
          <br />
          <Text type="secondary" className="text-xs">
            {instance.status} • {new Date(instance.created_at).toLocaleTimeString()}
          </Text>
        </div>
      ),
    }));
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
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <Title level={2} className="mb-2">AI Workflow Dashboard</Title>
          <Text type="secondary">Real-time monitoring of your workflow orchestration system</Text>
        </div>
        <Space>
          <Button 
            type="primary" 
            icon={<PlusOutlined />}
            onClick={() => navigate('/workflows')}
          >
            New Workflow
          </Button>
        </Space>
      </div>
      
      {/* Key Metrics */}
      <Row gutter={[24, 24]}>
        <Col xs={24} sm={12} lg={6}>
          <Card className="text-center hover:shadow-lg transition-shadow">
            <Statistic
              title="Total Workflows"
              value={stats.total}
              prefix={<RocketOutlined className="text-blue-500" />}
              valueStyle={{ color: '#1890ff', fontSize: '32px' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card className="text-center hover:shadow-lg transition-shadow">
            <Statistic
              title="Completed"
              value={stats.completed}
              prefix={<CheckCircleOutlined className="text-green-500" />}
              valueStyle={{ color: '#52c41a', fontSize: '32px' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card className="text-center hover:shadow-lg transition-shadow">
            <Statistic
              title="Running"
              value={stats.running}
              prefix={<SyncOutlined spin className="text-orange-500" />}
              valueStyle={{ color: '#fa8c16', fontSize: '32px' }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card className="text-center hover:shadow-lg transition-shadow">
            <Statistic
              title="Failed"
              value={stats.failed}
              prefix={<CloseCircleOutlined className="text-red-500" />}
              valueStyle={{ color: '#ff4d4f', fontSize: '32px' }}
            />
          </Card>
        </Col>
      </Row>
      
      {/* Performance & Activity */}
      <Row gutter={[24, 24]}>
        <Col xs={24} lg={12}>
          <Card
            title="Success Rate"
            extra={<Tag color="success">Live</Tag>}
            className="h-full"
          >
            <div className="text-center">
              <Progress
                type="dashboard"
                percent={Math.round(stats.successRate)}
                strokeColor={{
                  '0%': '#108ee9',
                  '100%': '#87d068',
                }}
                size={160}
              />
              <div className="mt-4">
                <Text type="secondary">
                  {stats.completed} successful out of {stats.total} total executions
                </Text>
              </div>
            </div>
          </Card>
        </Col>
        
        <Col xs={24} lg={12}>
          <Card title="Recent Activity" className="h-full">
            {getRecentActivity().length > 0 ? (
              <Timeline items={getRecentActivity()} />
            ) : (
              <div className="text-center py-8">
                <Text type="secondary">No recent activity</Text>
                <br />
                <Button 
                  type="link" 
                  onClick={() => navigate('/workflows')}
                >
                  Create your first workflow
                </Button>
              </div>
            )}
          </Card>
        </Col>
      </Row>
      
      {/* System Status */}
      <Card title="System Status" extra={<Tag color="success">All Systems Operational</Tag>}>
        <Row gutter={[16, 16]}>
          <Col xs={24} sm={12} md={8}>
            <div className="text-center p-4">
              <ApiOutlined className="text-3xl text-blue-500 mb-2" />
              <div>
                <Text strong className="block">API Server</Text>
                <Tag color="success">Online</Tag>
              </div>
            </div>
          </Col>
          <Col xs={24} sm={12} md={8}>
            <div className="text-center p-4">
              <CloudServerOutlined className="text-3xl text-green-500 mb-2" />
              <div>
                <Text strong className="block">MCP Services</Text>
                <Tag color="success">Connected</Tag>
              </div>
            </div>
          </Col>
          <Col xs={24} sm={12} md={8}>
            <div className="text-center p-4">
              <RocketOutlined className="text-3xl text-purple-500 mb-2" />
              <div>
                <Text strong className="block">AI Providers</Text>
                <Tag color="success">Available</Tag>
              </div>
            </div>
          </Col>
        </Row>
      </Card>
    </div>
  );
};

export default DashboardPage;