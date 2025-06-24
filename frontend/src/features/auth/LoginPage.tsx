import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Form, Input, Button, Card, Alert, Select, Typography } from 'antd';
import { UserOutlined, KeyOutlined, RocketOutlined } from '@ant-design/icons';
import { authStore } from '../../stores/authStore';

const { Title, Text } = Typography;
const { Option } = Select;

const LoginPage: React.FC = () => {
  const navigate = useNavigate();
  const { login, loginDemo, isLoading, error } = authStore();
  const [form] = Form.useForm();
  
  const handleLogin = async (values: { username: string; role: string }) => {
    try {
      await login(values.username, values.role);
      navigate('/dashboard');
    } catch (error) {
      // Error is handled in the store
    }
  };
  
  const quickDemoLogin = (username: string, role: string) => {
    try {
      loginDemo(username, role);
      navigate('/dashboard');
    } catch (error) {
      // Error handling is done in the store
    }
  };
  
  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 p-4">
      <Card
        className="w-full max-w-md shadow-2xl"
        bordered={false}
      >
        <div className="text-center mb-8">
          <RocketOutlined className="text-5xl text-blue-500 mb-4" />
          <Title level={2} className="mb-2">AI Workflow Engine</Title>
          <Text type="secondary">Enterprise AI Orchestration Platform</Text>
        </div>
        
        {error && (
          <Alert
            message={error}
            type="error"
            showIcon
            closable
            className="mb-4"
            onClose={() => authStore.setState({ error: null })}
          />
        )}
        
        <Form
          form={form}
          name="login"
          onFinish={handleLogin}
          layout="vertical"
          requiredMark={false}
        >
          <Form.Item
            name="username"
            label="Username"
            rules={[{ required: true, message: 'Please enter your username' }]}
          >
            <Input
              prefix={<UserOutlined />}
              placeholder="Enter username"
              size="large"
            />
          </Form.Item>
          
          <Form.Item
            name="role"
            label="Role"
            rules={[{ required: true, message: 'Please select a role' }]}
          >
            <Select
              placeholder="Select your role"
              size="large"
              prefix={<KeyOutlined />}
            >
              <Option value="admin">Admin</Option>
              <Option value="developer">Developer</Option>
              <Option value="analyst">Analyst</Option>
              <Option value="viewer">Viewer</Option>
            </Select>
          </Form.Item>
          
          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              loading={isLoading}
              size="large"
              block
            >
              Sign In
            </Button>
          </Form.Item>
        </Form>
        
        <div className="mt-6 pt-6 border-t">
          <Text type="secondary" className="block text-center mb-4">
            Quick Demo Access
          </Text>
          <div className="grid grid-cols-2 gap-2">
            <Button
              onClick={() => quickDemoLogin('demo_admin', 'admin')}
              className="text-xs"
            >
              Admin Demo
            </Button>
            <Button
              onClick={() => quickDemoLogin('demo_developer', 'developer')}
              className="text-xs"
            >
              Developer Demo
            </Button>
            <Button
              onClick={() => quickDemoLogin('demo_analyst', 'analyst')}
              className="text-xs"
            >
              Analyst Demo
            </Button>
            <Button
              onClick={() => quickDemoLogin('demo_viewer', 'viewer')}
              className="text-xs"
            >
              Viewer Demo
            </Button>
          </div>
        </div>
        
        <div className="mt-6 text-center">
          <Text type="secondary" className="text-xs">
            Demo Mode - No backend required. All features work with mock data.
          </Text>
        </div>
      </Card>
    </div>
  );
};

export default LoginPage;