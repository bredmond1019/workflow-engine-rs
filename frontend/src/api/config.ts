import axios, { type AxiosInstance, type InternalAxiosRequestConfig } from 'axios';
import { authStore } from '../stores/authStore';

// API Base URL - can be configured via environment variable
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

// Create axios instance
export const apiClient: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor to add auth token
apiClient.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    const token = authStore.getState().token;
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor to handle errors
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      // Token expired or invalid
      authStore.getState().logout();
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// API endpoints
export const API_ENDPOINTS = {
  // Auth
  auth: {
    token: '/auth/token',
    verify: '/auth/verify',
  },
  
  // Workflows
  workflows: {
    trigger: '/api/v1/workflows/trigger',
    status: (id: string) => `/api/v1/workflows/status/${id}`,
    instances: '/api/v1/workflows/instances',
    available: '/api/v1/workflows/available',
  },
  
  // Templates
  templates: {
    list: '/api/v1/templates',
    search: '/api/v1/templates/search',
    get: (id: string) => `/api/v1/templates/${id}`,
    categories: '/api/v1/templates/categories',
    tags: '/api/v1/templates/tags',
    trigger: '/api/v1/templates/trigger',
  },
  
  // Health
  health: {
    basic: '/health',
    detailed: '/health/detailed',
  },
  
  // Metrics
  metrics: '/metrics',
  
  // Service Registry
  registry: {
    services: '/api/v1/registry/services',
    register: '/api/v1/registry/register',
    deregister: (id: string) => `/api/v1/registry/deregister/${id}`,
  },
};