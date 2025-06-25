import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Table,
  Button,
  Space,
  Tag,
  Drawer,
  Form,
  Input,
  Select,
  Card,
  Typography,
  Row,
  Col,
  Spin,
  message,
  Progress,
} from 'antd';
import {
  PlusOutlined,
  PlayCircleOutlined,
  EyeOutlined,
  ReloadOutlined,
  FilterOutlined,
} from '@ant-design/icons';
import { workflowStore } from '../../stores/workflowStore';
import type { WorkflowInstance } from '../../types';
import { WorkflowStatus } from '../../types';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

dayjs.extend(relativeTime);

const { Title, Text } = Typography;
const { Option } = Select;
const { TextArea } = Input;

const WorkflowsPage: React.FC = () => {
  const navigate = useNavigate();
  const {
    instances,
    availableWorkflows,
    fetchAllInstances,
    fetchAvailableWorkflows,
    triggerWorkflow,
    isLoading,
    startPolling,
    stopPolling,
  } = workflowStore();
  
  const [form] = Form.useForm();
  const [drawerVisible, setDrawerVisible] = useState(false);
  const [selectedWorkflow, setSelectedWorkflow] = useState<string>('');
  const [filterStatus, setFilterStatus] = useState<WorkflowStatus | 'all'>('all');
  const [refreshing, setRefreshing] = useState(false);
  
  useEffect(() => {
    loadData();
    startPolling();
    
    return () => {
      stopPolling();
    };
  }, []);
  
  const loadData = async () => {
    await Promise.all([
      fetchAllInstances(),
      fetchAvailableWorkflows(),
    ]);
  };
  
  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await loadData();
    } finally {
      setRefreshing(false);
    }
  };
  
  const handleTriggerWorkflow = async (values: any) => {
    try {
      const instanceId = await triggerWorkflow(
        selectedWorkflow,
        JSON.parse(values.inputs || '{}'),
        values.config ? JSON.parse(values.config) : undefined
      );
      
      message.success('Workflow triggered successfully!');
      setDrawerVisible(false);
      form.resetFields();
      
      // Navigate to the workflow detail page
      navigate(`/workflows/${instanceId}`);
    } catch (error: any) {
      message.error(error.message || 'Failed to trigger workflow');
    }
  };
  
  const getStatusColor = (status: WorkflowStatus) => {
    switch (status) {
      case WorkflowStatus.Completed:
        return 'success';
      case WorkflowStatus.Running:
        return 'processing';
      case WorkflowStatus.Failed:
        return 'error';
      case WorkflowStatus.Created:
        return 'default';
      case WorkflowStatus.Cancelled:
        return 'warning';
      default:
        return 'default';
    }
  };
  
  const getStatusIcon = (status: WorkflowStatus) => {
    switch (status) {
      case WorkflowStatus.Running:
        return <Spin size="small" />;
      default:
        return null;
    }
  };
  
  const filteredInstances = Array.from(instances.values()).filter(instance => 
    filterStatus === 'all' || instance.status === filterStatus
  );
  
  const columns = [
    {
      title: 'Workflow',
      dataIndex: 'workflow_name',
      key: 'workflow_name',
      render: (name: string, record: WorkflowInstance) => (
        <div>
          <Text strong className="block">
            {name.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
          </Text>
          <Text code className="text-xs">{record.instance_id.slice(0, 12)}...</Text>
        </div>
      ),
    },
    {
      title: 'Status & Progress',
      key: 'status_progress',
      render: (record: WorkflowInstance) => (
        <div>
          <div className="mb-2">
            <Tag color={getStatusColor(record.status)} icon={getStatusIcon(record.status)}>
              {record.status}
            </Tag>
          </div>
          <Progress
            percent={record.progress?.percentage || 0}
            size="small"
            status={
              record.status === WorkflowStatus.Failed ? 'exception' :
              record.progress?.percentage === 100 ? 'success' : 'active'
            }
            format={(percent) => `${percent}%`}
          />
          {record.progress && (
            <Text className="text-xs text-gray-500">
              {record.progress.completed_steps}/{record.progress.total_steps} steps
            </Text>
          )}
        </div>
      ),
    },
    {
      title: 'Timing',
      key: 'timing',
      render: (record: WorkflowInstance) => {
        const duration = (() => {
          if (record.completed_at && record.started_at) {
            const dur = dayjs(record.completed_at).diff(dayjs(record.started_at), 'second');
            return `${dur}s`;
          }
          if (record.started_at) {
            const dur = dayjs().diff(dayjs(record.started_at), 'second');
            return `${dur}s (running)`;
          }
          return '-';
        })();
        
        return (
          <div>
            <Text className="block">{duration}</Text>
            <Text type="secondary" className="text-xs">
              {dayjs(record.created_at).fromNow()}
            </Text>
          </div>
        );
      },
    },
    {
      title: 'Actions',
      key: 'actions',
      width: 100,
      render: (record: WorkflowInstance) => (
        <Button
          type="primary"
          size="small"
          icon={<EyeOutlined />}
          onClick={() => navigate(`/workflows/${record.instance_id}`)}
        >
          View
        </Button>
      ),
    },
  ];
  
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <Title level={2}>Workflow Instances</Title>
          <Text type="secondary">
            Manage and monitor your AI workflow executions
          </Text>
        </div>
        <Space>
          <Button
            icon={<ReloadOutlined />}
            loading={refreshing}
            onClick={handleRefresh}
          >
            Refresh
          </Button>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => setDrawerVisible(true)}
          >
            Trigger Workflow
          </Button>
        </Space>
      </div>
      
      {/* Quick Stats */}
      <Row gutter={[16, 16]}>
        <Col xs={12} sm={6}>
          <Card className="text-center">
            <div className="text-2xl font-bold text-blue-600">
              {instances.size}
            </div>
            <Text type="secondary">Total</Text>
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card className="text-center">
            <div className="text-2xl font-bold text-orange-600">
              {filteredInstances.filter(i => i.status === WorkflowStatus.Running).length}
            </div>
            <Text type="secondary">Running</Text>
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {filteredInstances.filter(i => i.status === WorkflowStatus.Completed).length}
            </div>
            <Text type="secondary">Completed</Text>
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card className="text-center">
            <div className="text-2xl font-bold text-red-600">
              {filteredInstances.filter(i => i.status === WorkflowStatus.Failed).length}
            </div>
            <Text type="secondary">Failed</Text>
          </Card>
        </Col>
      </Row>
      
      {/* Filters */}
      <Card>
        <div className="flex items-center justify-between">
          <Space>
            <FilterOutlined />
            <Text>Filter by status:</Text>
            <Select
              value={filterStatus}
              onChange={setFilterStatus}
              style={{ width: 120 }}
            >
              <Option value="all">All</Option>
              <Option value={WorkflowStatus.Created}>Created</Option>
              <Option value={WorkflowStatus.Running}>Running</Option>
              <Option value={WorkflowStatus.Completed}>Completed</Option>
              <Option value={WorkflowStatus.Failed}>Failed</Option>
              <Option value={WorkflowStatus.Cancelled}>Cancelled</Option>
            </Select>
          </Space>
          <Text type="secondary">
            {filteredInstances.length} of {instances.size} workflows
          </Text>
        </div>
      </Card>
      
      {/* Workflow Instances Table */}
      <Card>
        <Table
          columns={columns}
          dataSource={filteredInstances}
          rowKey="instance_id"
          loading={isLoading}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} instances`,
          }}
        />
      </Card>
      
      {/* Trigger Workflow Drawer */}
      <Drawer
        title="Trigger New Workflow"
        width={600}
        onClose={() => setDrawerVisible(false)}
        open={drawerVisible}
        extra={
          <Space>
            <Button onClick={() => setDrawerVisible(false)}>Cancel</Button>
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              loading={isLoading}
              onClick={() => form.submit()}
            >
              Trigger
            </Button>
          </Space>
        }
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={handleTriggerWorkflow}
          initialValues={{
            inputs: JSON.stringify({
              "query": "Analyze customer feedback",
              "priority": "high"
            }, null, 2)
          }}
        >
          <Form.Item
            name="workflow"
            label="Select Workflow"
            rules={[{ required: true, message: 'Please select a workflow' }]}
          >
            <Select
              placeholder="Choose a workflow to execute"
              onChange={setSelectedWorkflow}
              showSearch
              size="large"
            >
              {availableWorkflows.map(workflow => (
                <Option key={workflow} value={workflow}>
                  <div>
                    <Text strong>{workflow.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}</Text>
                    <br />
                    <Text type="secondary" className="text-xs">{workflow}</Text>
                  </div>
                </Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="inputs"
            label="Input Data"
            help="Provide workflow input as JSON"
            rules={[
              { required: true, message: 'Please provide input data' },
              {
                validator: (_, value) => {
                  if (!value) return Promise.resolve();
                  try {
                    JSON.parse(value);
                    return Promise.resolve();
                  } catch {
                    return Promise.reject('Invalid JSON format');
                  }
                },
              },
            ]}
          >
            <TextArea
              rows={8}
              placeholder={JSON.stringify({
                "query": "Customer inquiry about delayed order",
                "priority": "high",
                "customer_id": "12345"
              }, null, 2)}
              className="font-mono text-sm"
            />
          </Form.Item>
          
          <Form.Item
            name="config"
            label="Advanced Configuration (Optional)"
            help="Override default workflow settings"
          >
            <TextArea
              rows={4}
              placeholder={JSON.stringify({
                "timeout": 300,
                "retries": 3
              }, null, 2)}
              className="font-mono text-sm"
            />
          </Form.Item>
        </Form>
      </Drawer>
    </div>
  );
};

export default WorkflowsPage;