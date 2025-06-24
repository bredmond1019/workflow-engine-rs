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
  } = workflowStore();
  
  const [form] = Form.useForm();
  const [drawerVisible, setDrawerVisible] = useState(false);
  const [selectedWorkflow, setSelectedWorkflow] = useState<string>('');
  const [filterStatus, setFilterStatus] = useState<WorkflowStatus | 'all'>('all');
  const [refreshing, setRefreshing] = useState(false);
  
  useEffect(() => {
    loadData();
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
      title: 'Instance ID',
      dataIndex: 'instance_id',
      key: 'instance_id',
      width: 120,
      render: (id: string) => (
        <Text code className="text-xs">{id.slice(0, 8)}...</Text>
      ),
    },
    {
      title: 'Workflow',
      dataIndex: 'workflow_name',
      key: 'workflow_name',
      render: (name: string) => <Text strong>{name}</Text>,
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status: WorkflowStatus) => (
        <Tag color={getStatusColor(status)} icon={getStatusIcon(status)}>
          {status}
        </Tag>
      ),
    },
    {
      title: 'Progress',
      dataIndex: 'progress',
      key: 'progress',
      width: 120,
      render: (progress: any) => (
        <Progress
          percent={progress?.percentage || 0}
          size="small"
          status={progress?.percentage === 100 ? 'success' : 'active'}
          format={(percent) => `${percent}%`}
        />
      ),
    },
    {
      title: 'Created',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => (
        <Text type="secondary">{dayjs(date).fromNow()}</Text>
      ),
    },
    {
      title: 'Duration',
      key: 'duration',
      render: (record: WorkflowInstance) => {
        if (record.completed_at && record.started_at) {
          const duration = dayjs(record.completed_at).diff(dayjs(record.started_at), 'second');
          return <Text>{duration}s</Text>;
        }
        if (record.started_at) {
          const duration = dayjs().diff(dayjs(record.started_at), 'second');
          return <Text type="secondary">{duration}s (running)</Text>;
        }
        return <Text type="secondary">-</Text>;
      },
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (record: WorkflowInstance) => (
        <Space>
          <Button
            type="link"
            icon={<EyeOutlined />}
            onClick={() => navigate(`/workflows/${record.instance_id}`)}
          >
            View
          </Button>
        </Space>
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
        <Col span={6}>
          <Card>
            <Text type="secondary">Total Instances</Text>
            <div className="text-2xl font-bold text-blue-600">
              {instances.size}
            </div>
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Text type="secondary">Running</Text>
            <div className="text-2xl font-bold text-orange-600">
              {filteredInstances.filter(i => i.status === WorkflowStatus.Running).length}
            </div>
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Text type="secondary">Completed</Text>
            <div className="text-2xl font-bold text-green-600">
              {filteredInstances.filter(i => i.status === WorkflowStatus.Completed).length}
            </div>
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Text type="secondary">Failed</Text>
            <div className="text-2xl font-bold text-red-600">
              {filteredInstances.filter(i => i.status === WorkflowStatus.Failed).length}
            </div>
          </Card>
        </Col>
      </Row>
      
      {/* Filters */}
      <Card>
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
            >
              {availableWorkflows.map(workflow => (
                <Option key={workflow} value={workflow}>
                  {workflow}
                </Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="inputs"
            label="Input Data (JSON)"
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
              rows={6}
              placeholder={JSON.stringify({
                "query": "Example query",
                "model": "gpt-4",
                "max_tokens": 1000
              }, null, 2)}
            />
          </Form.Item>
          
          <Form.Item
            name="config"
            label="Configuration Overrides (JSON, Optional)"
            rules={[
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
              rows={4}
              placeholder={JSON.stringify({
                "timeout": 300,
                "retries": 3,
                "continue_on_error": false
              }, null, 2)}
            />
          </Form.Item>
        </Form>
      </Drawer>
    </div>
  );
};

export default WorkflowsPage;