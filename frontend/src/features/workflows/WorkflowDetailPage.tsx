import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Card,
  Typography,
  Steps,
  Button,
  Tag,
  Alert,
  Descriptions,
  Spin,
  Row,
  Col,
  Progress,
  Divider,
  Badge,
} from 'antd';
import {
  ArrowLeftOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  SyncOutlined,
  ClockCircleOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons';
import { workflowStore } from '../../stores/workflowStore';
import type { WorkflowInstance } from '../../types';
import { WorkflowStatus } from '../../types';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import duration from 'dayjs/plugin/duration';

dayjs.extend(relativeTime);
dayjs.extend(duration);

const { Title, Text, Paragraph } = Typography;

const WorkflowDetailPage: React.FC = () => {
  const { instanceId } = useParams<{ instanceId: string }>();
  const navigate = useNavigate();
  const {
    instances,
    fetchWorkflowStatus,
    startPolling,
    stopPolling,
  } = workflowStore();
  
  const [loading, setLoading] = useState(false);
  const [instance, setInstance] = useState<WorkflowInstance | null>(null);
  
  useEffect(() => {
    if (!instanceId) return;
    
    // Get instance from store or fetch it
    const existingInstance = instances.get(instanceId);
    if (existingInstance) {
      setInstance(existingInstance);
    }
    
    // Always fetch latest status
    loadWorkflowStatus();
    
    // Start polling for real-time updates
    startPolling();
    
    return () => {
      stopPolling();
    };
  }, [instanceId]);
  
  useEffect(() => {
    // Update local instance when store updates
    if (instanceId && instances.has(instanceId)) {
      setInstance(instances.get(instanceId)!);
    }
  }, [instances, instanceId]);
  
  const loadWorkflowStatus = async () => {
    if (!instanceId) return;
    
    setLoading(true);
    try {
      await fetchWorkflowStatus(instanceId);
    } catch (error) {
      console.error('Failed to fetch workflow status:', error);
    } finally {
      setLoading(false);
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
      case WorkflowStatus.Completed:
        return <CheckCircleOutlined />;
      case WorkflowStatus.Running:
        return <SyncOutlined spin />;
      case WorkflowStatus.Failed:
        return <CloseCircleOutlined />;
      case WorkflowStatus.Created:
        return <ClockCircleOutlined />;
      case WorkflowStatus.Cancelled:
        return <CloseCircleOutlined />;
      default:
        return <InfoCircleOutlined />;
    }
  };
  
  const getDuration = () => {
    if (!instance) return '-';
    
    if (instance.completed_at && instance.started_at) {
      const duration = dayjs(instance.completed_at).diff(dayjs(instance.started_at), 'second');
      return `${duration}s`;
    }
    
    if (instance.started_at) {
      const duration = dayjs().diff(dayjs(instance.started_at), 'second');
      return `${duration}s (running)`;
    }
    
    return '-';
  };
  
  const renderStepProgress = () => {
    if (!instance || !instance.steps) return null;
    
    const stepEntries = Object.entries(instance.steps);
    if (stepEntries.length === 0) {
      return (
        <Card title="Workflow Steps" className="mb-6">
          <div className="text-center py-8">
            <Spin size="large" />
            <div className="mt-4">
              <Text type="secondary">Initializing workflow steps...</Text>
            </div>
          </div>
        </Card>
      );
    }
    
    return (
      <Card title="Workflow Steps" className="mb-6">
        <Steps
          direction="vertical"
          current={-1}
          items={stepEntries.map(([stepId, stepStatus]) => ({
            title: stepId.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
            description: (
              <div className="mt-2">
                <div className="flex items-center space-x-2 mb-2">
                  <Badge
                    status={
                      stepStatus.status === 'Completed' ? 'success' :
                      stepStatus.status === 'Running' ? 'processing' :
                      stepStatus.status === 'Failed' ? 'error' : 'default'
                    }
                    text={stepStatus.status}
                  />
                  {stepStatus.attempt > 1 && (
                    <Tag color="orange">Attempt {stepStatus.attempt}</Tag>
                  )}
                </div>
                
                {stepStatus.started_at && (
                  <Text type="secondary" className="block">
                    Started: {dayjs(stepStatus.started_at).format('HH:mm:ss')}
                  </Text>
                )}
                
                {stepStatus.completed_at && (
                  <Text type="secondary" className="block">
                    Completed: {dayjs(stepStatus.completed_at).format('HH:mm:ss')}
                  </Text>
                )}
                
                {stepStatus.error && (
                  <Alert
                    message="Step Error"
                    description={stepStatus.error}
                    type="error"
                    className="mt-2"
                  />
                )}
                
                {stepStatus.output && (
                  <Card size="small" className="mt-2">
                    <Text strong>Output:</Text>
                    <Paragraph className="mt-1 mb-0">
                      <pre className="text-xs overflow-x-auto">
                        {typeof stepStatus.output === 'string' 
                          ? stepStatus.output 
                          : JSON.stringify(stepStatus.output, null, 2)
                        }
                      </pre>
                    </Paragraph>
                  </Card>
                )}
              </div>
            ),
            status: 
              stepStatus.status === 'Completed' ? 'finish' :
              stepStatus.status === 'Running' ? 'process' :
              stepStatus.status === 'Failed' ? 'error' : 'wait',
            icon: 
              stepStatus.status === 'Completed' ? <CheckCircleOutlined /> :
              stepStatus.status === 'Running' ? <SyncOutlined spin /> :
              stepStatus.status === 'Failed' ? <CloseCircleOutlined /> :
              <ClockCircleOutlined />,
          }))}
        />
      </Card>
    );
  };
  
  if (!instanceId) {
    return (
      <div className="text-center py-16">
        <Alert message="Invalid workflow instance ID" type="error" />
      </div>
    );
  }
  
  if (loading && !instance) {
    return (
      <div className="text-center py-16">
        <Spin size="large" tip="Loading workflow details..." />
      </div>
    );
  }
  
  if (!instance) {
    return (
      <div className="text-center py-16">
        <Alert 
          message="Workflow instance not found" 
          type="warning"
          action={
            <Button size="small" onClick={() => navigate('/workflows')}>
              Back to Workflows
            </Button>
          }
        />
      </div>
    );
  }
  
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:justify-between sm:items-start space-y-4 sm:space-y-0">
        <div className="flex-1">
          <Button 
            type="text" 
            icon={<ArrowLeftOutlined />}
            onClick={() => navigate('/workflows')}
            className="mb-2"
          >
            Back to Workflows
          </Button>
          <Title level={2} className="mb-2 text-lg sm:text-xl lg:text-2xl">
            {instance.workflow_name.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
          </Title>
          <div className="flex flex-col sm:flex-row sm:items-center gap-2">
            <Text code className="text-xs sm:text-sm break-all">{instance.instance_id}</Text>
            <Tag color={getStatusColor(instance.status)} icon={getStatusIcon(instance.status)}>
              {instance.status}
            </Tag>
          </div>
        </div>
        <Button 
          icon={<SyncOutlined />}
          loading={loading}
          onClick={loadWorkflowStatus}
          className="self-start sm:self-auto"
        >
          <span className="hidden sm:inline">Refresh</span>
        </Button>
      </div>
      
      {/* Overview Cards */}
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Text type="secondary">Status</Text>
            <div className="text-lg font-semibold">
              <Tag color={getStatusColor(instance.status)} icon={getStatusIcon(instance.status)}>
                {instance.status}
              </Tag>
            </div>
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Text type="secondary">Progress</Text>
            <div className="text-lg font-semibold">
              <Progress 
                percent={instance.progress?.percentage || 0}
                size="small"
                status={instance.status === WorkflowStatus.Failed ? 'exception' : 'active'}
              />
              {instance.progress && (
                <Text className="text-xs">
                  {instance.progress.completed_steps}/{instance.progress.total_steps} steps
                </Text>
              )}
            </div>
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Text type="secondary">Duration</Text>
            <div className="text-lg font-semibold text-blue-600">
              {getDuration()}
            </div>
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Text type="secondary">Created</Text>
            <div className="text-lg font-semibold text-gray-600">
              {dayjs(instance.created_at).fromNow()}
            </div>
          </Card>
        </Col>
      </Row>
      
      {/* Workflow Information */}
      <Card title="Workflow Information">
        <Descriptions column={2}>
          <Descriptions.Item label="Instance ID">
            <Text code>{instance.instance_id}</Text>
          </Descriptions.Item>
          <Descriptions.Item label="Workflow Name">
            {instance.workflow_name}
          </Descriptions.Item>
          <Descriptions.Item label="Created At">
            {dayjs(instance.created_at).format('YYYY-MM-DD HH:mm:ss')}
          </Descriptions.Item>
          <Descriptions.Item label="Started At">
            {instance.started_at 
              ? dayjs(instance.started_at).format('YYYY-MM-DD HH:mm:ss')
              : '-'
            }
          </Descriptions.Item>
          <Descriptions.Item label="Completed At">
            {instance.completed_at 
              ? dayjs(instance.completed_at).format('YYYY-MM-DD HH:mm:ss')
              : '-'
            }
          </Descriptions.Item>
          <Descriptions.Item label="Duration">
            {getDuration()}
          </Descriptions.Item>
        </Descriptions>
        
        {instance.inputs && (
          <>
            <Divider />
            <div>
              <Text strong>Input Data:</Text>
              <Card size="small" className="mt-2">
                <pre className="text-xs overflow-x-auto">
                  {JSON.stringify(instance.inputs, null, 2)}
                </pre>
              </Card>
            </div>
          </>
        )}
        
        {instance.outputs && (
          <>
            <Divider />
            <div>
              <Text strong>Final Output:</Text>
              <Card size="small" className="mt-2">
                <pre className="text-xs overflow-x-auto">
                  {JSON.stringify(instance.outputs, null, 2)}
                </pre>
              </Card>
            </div>
          </>
        )}
        
        {instance.error && (
          <>
            <Divider />
            <Alert
              message="Workflow Error"
              description={
                <div>
                  <Text strong>Message:</Text> {instance.error.message}<br />
                  <Text strong>Code:</Text> {instance.error.code}<br />
                  {instance.error.step_id && (
                    <>
                      <Text strong>Failed Step:</Text> {instance.error.step_id}<br />
                    </>
                  )}
                  {instance.error.details && (
                    <>
                      <Text strong>Details:</Text>
                      <pre className="text-xs mt-1">
                        {JSON.stringify(instance.error.details, null, 2)}
                      </pre>
                    </>
                  )}
                </div>
              }
              type="error"
              className="mt-2"
            />
          </>
        )}
      </Card>
      
      {/* Step Progress */}
      {renderStepProgress()}
    </div>
  );
};

export default WorkflowDetailPage;