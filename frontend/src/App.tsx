import React, { useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider, theme } from 'antd';
import { authStore } from './stores/authStore';

// Layout Components
import MainLayout from './components/Layout/MainLayout';
import ProtectedRoute from './components/Auth/ProtectedRoute';

// Feature Pages
import LoginPage from './features/auth/LoginPage';
import DashboardPage from './features/dashboard/DashboardPage';
import WorkflowsPage from './features/workflows/WorkflowsPage';
import WorkflowDetailPage from './features/workflows/WorkflowDetailPage';
import TemplatesPage from './features/templates/TemplatesPage';
import DemosPage from './features/demos/DemosPage';
import MonitoringPage from './features/monitoring/MonitoringPage';


const App: React.FC = () => {
  const { verifyToken } = authStore();
  
  useEffect(() => {
    // Verify token on app load
    verifyToken();
  }, [verifyToken]);
  
  return (
    <ConfigProvider
      theme={{
        algorithm: theme.defaultAlgorithm,
        token: {
          colorPrimary: '#1890ff',
          borderRadius: 6,
        },
      }}
    >
      <Router>
        <Routes>
          {/* Public Routes */}
          <Route path="/login" element={<LoginPage />} />
          
          {/* Protected Routes */}
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <MainLayout />
              </ProtectedRoute>
            }
          >
            <Route index element={<Navigate to="/dashboard" replace />} />
            <Route path="dashboard" element={<DashboardPage />} />
            <Route path="workflows" element={<WorkflowsPage />} />
            <Route path="workflows/:instanceId" element={<WorkflowDetailPage />} />
            <Route path="templates" element={<TemplatesPage />} />
            <Route path="demos" element={<DemosPage />} />
            <Route path="monitoring" element={<MonitoringPage />} />
          </Route>
          
          {/* Catch all */}
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </Router>
    </ConfigProvider>
  );
};

export default App;