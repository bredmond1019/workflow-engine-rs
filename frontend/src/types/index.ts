// API Types
export interface AuthToken {
  access_token: string;
  token_type: string;
  expires_in: number;
}

export interface User {
  sub: string;
  role: string;
  exp?: number;
  iat?: number;
}

// Workflow Types
export interface WorkflowTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  tags: string[];
  input_schema?: any;
  preview?: string;
}

export interface WorkflowInstance {
  instance_id: string;
  workflow_name: string;
  status: WorkflowStatus;
  created_at: string;
  started_at?: string;
  completed_at?: string;
  inputs: any;
  outputs?: any;
  error?: WorkflowError;
  progress: WorkflowProgress;
  steps: Record<string, StepStatus>;
}

export const WorkflowStatus = {
  Created: 'Created',
  Running: 'Running',
  Completed: 'Completed',
  Failed: 'Failed',
  Cancelled: 'Cancelled'
} as const;

export type WorkflowStatus = typeof WorkflowStatus[keyof typeof WorkflowStatus];

export interface WorkflowError {
  message: string;
  code: string;
  step_id?: string;
  details?: any;
}

export interface WorkflowProgress {
  total_steps: number;
  completed_steps: number;
  failed_steps: number;
  running_steps: number;
  percentage: number;
}

export interface StepStatus {
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Skipped';
  output?: any;
  started_at?: string;
  completed_at?: string;
  error?: string;
  attempt: number;
}

// Health Check Types
export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  uptime_seconds: number;
  version: string;
}

export interface DetailedHealthStatus extends HealthStatus {
  checks: HealthChecks;
  system_info: SystemInfo;
}

export interface HealthChecks {
  database: ComponentHealth;
  mcp_servers: Record<string, MCPServerHealth>;
  redis?: ComponentHealth;
}

export interface ComponentHealth {
  status: 'healthy' | 'unhealthy';
  latency_ms?: number;
  error?: string;
}

export interface MCPServerHealth extends ComponentHealth {
  url: string;
  available_tools?: string[];
}

export interface SystemInfo {
  memory: MemoryInfo;
  disk: DiskInfo[];
  process: ProcessInfo;
}

export interface MemoryInfo {
  total_mb: number;
  used_mb: number;
  free_mb: number;
  usage_percent: number;
}

export interface DiskInfo {
  mount_point: string;
  total_gb: number;
  used_gb: number;
  free_gb: number;
  usage_percent: number;
}

export interface ProcessInfo {
  pid: number;
  cpu_percent: number;
  memory_mb: number;
  threads: number;
}

// Service Registry Types
export interface ServiceInfo {
  id: string;
  name: string;
  version: string;
  status: 'healthy' | 'degraded' | 'unhealthy';
  endpoints: string[];
  capabilities: string[];
  metadata?: Record<string, any>;
}

// Metrics Types
export interface Metric {
  name: string;
  value: number;
  timestamp: string;
  labels?: Record<string, string>;
}

export interface MetricSeries {
  name: string;
  data: Array<{
    timestamp: string;
    value: number;
  }>;
}