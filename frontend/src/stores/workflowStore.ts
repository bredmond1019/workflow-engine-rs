import { create } from 'zustand';
import type { WorkflowInstance, WorkflowTemplate } from '../types';
import { apiClient, API_ENDPOINTS } from '../api/config';

interface WorkflowState {
  // State
  templates: WorkflowTemplate[];
  instances: Map<string, WorkflowInstance>;
  availableWorkflows: string[];
  categories: string[];
  tags: string[];
  isLoading: boolean;
  error: string | null;
  
  // Actions
  fetchTemplates: () => Promise<void>;
  fetchTemplate: (id: string) => Promise<WorkflowTemplate | null>;
  fetchCategories: () => Promise<void>;
  fetchTags: () => Promise<void>;
  fetchAvailableWorkflows: () => Promise<void>;
  triggerWorkflow: (workflowName: string, inputs: any, config?: any) => Promise<string>;
  triggerFromTemplate: (templateId: string, inputs: any, config?: any) => Promise<string>;
  fetchWorkflowStatus: (instanceId: string) => Promise<WorkflowInstance>;
  fetchAllInstances: () => Promise<void>;
  updateInstance: (instanceId: string, instance: WorkflowInstance) => void;
  setError: (error: string | null) => void;
}

// Create some initial demo instances for showcase
const createDemoInstances = (): Map<string, WorkflowInstance> => {
  const instances = new Map<string, WorkflowInstance>();
  
  const demoData = [
    {
      id: 'demo-completed-1',
      name: 'customer_care_workflow',
      status: 'Completed' as any,
      created: new Date(Date.now() - 3600000).toISOString(), // 1 hour ago
      completed: new Date(Date.now() - 3590000).toISOString(), // 59 minutes ago
    },
    {
      id: 'demo-completed-2', 
      name: 'research_to_documentation',
      status: 'Completed' as any,
      created: new Date(Date.now() - 7200000).toISOString(), // 2 hours ago
      completed: new Date(Date.now() - 7080000).toISOString(), // 1h 58m ago
    },
    {
      id: 'demo-running-1',
      name: 'knowledge_base_workflow',
      status: 'Running' as any,
      created: new Date(Date.now() - 300000).toISOString(), // 5 minutes ago
      started: new Date(Date.now() - 290000).toISOString(), // 4m 50s ago
    }
  ];
  
  demoData.forEach(demo => {
    instances.set(demo.id, {
      instance_id: demo.id,
      workflow_name: demo.name,
      status: demo.status,
      created_at: demo.created,
      started_at: demo.started || demo.created,
      completed_at: demo.completed,
      inputs: { demo: true },
      progress: {
        total_steps: 3,
        completed_steps: demo.status === 'Completed' ? 3 : 2,
        failed_steps: 0,
        running_steps: demo.status === 'Running' ? 1 : 0,
        percentage: demo.status === 'Completed' ? 100 : 67,
      },
      steps: {},
    });
  });
  
  return instances;
};

export const workflowStore = create<WorkflowState>((set, get) => ({
  templates: [],
  instances: createDemoInstances(),
  availableWorkflows: [],
  categories: [],
  tags: [],
  isLoading: false,
  error: null,
  
  fetchTemplates: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await apiClient.get(API_ENDPOINTS.templates.list);
      set({ templates: response.data.templates, isLoading: false });
    } catch (error: any) {
      set({
        isLoading: false,
        error: error.response?.data?.message || 'Failed to fetch templates',
      });
    }
  },
  
  fetchTemplate: async (id: string) => {
    try {
      const response = await apiClient.get(API_ENDPOINTS.templates.get(id));
      return response.data;
    } catch (error) {
      console.error('Failed to fetch template:', error);
      return null;
    }
  },
  
  fetchCategories: async () => {
    try {
      const response = await apiClient.get(API_ENDPOINTS.templates.categories);
      set({ categories: response.data.categories });
    } catch (error) {
      console.error('Failed to fetch categories:', error);
    }
  },
  
  fetchTags: async () => {
    try {
      const response = await apiClient.get(API_ENDPOINTS.templates.tags);
      set({ tags: response.data.tags });
    } catch (error) {
      console.error('Failed to fetch tags:', error);
    }
  },
  
  fetchAvailableWorkflows: async () => {
    try {
      const response = await apiClient.get(API_ENDPOINTS.workflows.available);
      set({ availableWorkflows: response.data.workflows });
    } catch (error) {
      console.warn('API not available, using demo workflows');
      // Set demo workflows
      set({ 
        availableWorkflows: [
          'customer_care_workflow',
          'research_to_documentation', 
          'knowledge_base_workflow'
        ]
      });
    }
  },
  
  triggerWorkflow: async (workflowName: string, inputs: any, config?: any) => {
    set({ isLoading: true, error: null });
    try {
      const response = await apiClient.post(API_ENDPOINTS.workflows.trigger, {
        workflow_name: workflowName,
        inputs,
        config,
      });
      
      const { instance_id, ...instanceData } = response.data;
      
      // Create initial instance
      const instance: WorkflowInstance = {
        instance_id,
        workflow_name: instanceData.workflow_name,
        status: instanceData.status,
        created_at: instanceData.created_at,
        inputs,
        progress: {
          total_steps: 0,
          completed_steps: 0,
          failed_steps: 0,
          running_steps: 0,
          percentage: 0,
        },
        steps: {},
      };
      
      get().updateInstance(instance_id, instance);
      set({ isLoading: false });
      
      return instance_id;
    } catch (error: any) {
      // Fallback to demo mode if API is not available
      console.warn('API not available, using demo workflow execution');
      
      const instance_id = `demo-${Date.now()}`;
      const instance: WorkflowInstance = {
        instance_id,
        workflow_name: workflowName,
        status: 'Created' as any,
        created_at: new Date().toISOString(),
        started_at: new Date().toISOString(),
        inputs,
        progress: {
          total_steps: 3,
          completed_steps: 0,
          failed_steps: 0,
          running_steps: 1,
          percentage: 0,
        },
        steps: {},
      };
      
      get().updateInstance(instance_id, instance);
      set({ isLoading: false });
      
      return instance_id;
    }
  },
  
  triggerFromTemplate: async (templateId: string, inputs: any, config?: any) => {
    set({ isLoading: true, error: null });
    try {
      const response = await apiClient.post(API_ENDPOINTS.templates.trigger, {
        template_id: templateId,
        inputs,
        config,
      });
      
      const { instance_id, ...instanceData } = response.data;
      
      // Create initial instance
      const instance: WorkflowInstance = {
        instance_id,
        workflow_name: instanceData.workflow_name,
        status: instanceData.status,
        created_at: instanceData.created_at,
        inputs,
        progress: {
          total_steps: 0,
          completed_steps: 0,
          failed_steps: 0,
          running_steps: 0,
          percentage: 0,
        },
        steps: {},
      };
      
      get().updateInstance(instance_id, instance);
      set({ isLoading: false });
      
      return instance_id;
    } catch (error: any) {
      set({
        isLoading: false,
        error: error.response?.data?.message || 'Failed to trigger workflow from template',
      });
      throw error;
    }
  },
  
  fetchWorkflowStatus: async (instanceId: string) => {
    try {
      const response = await apiClient.get(API_ENDPOINTS.workflows.status(instanceId));
      const instance = response.data as WorkflowInstance;
      get().updateInstance(instanceId, instance);
      return instance;
    } catch (error: any) {
      throw error;
    }
  },
  
  fetchAllInstances: async () => {
    try {
      const response = await apiClient.get(API_ENDPOINTS.workflows.instances);
      const instances = response.data.instances;
      
      // Convert to Map
      const instanceMap = new Map<string, WorkflowInstance>();
      instances.forEach((inst: any) => {
        instanceMap.set(inst.instance_id, {
          instance_id: inst.instance_id,
          workflow_name: inst.workflow_name,
          status: inst.status,
          created_at: inst.created_at,
          inputs: {},
          progress: {
            total_steps: 0,
            completed_steps: 0,
            failed_steps: 0,
            running_steps: 0,
            percentage: 0,
          },
          steps: {},
        });
      });
      
      set({ instances: instanceMap });
    } catch (error) {
      console.error('Failed to fetch instances:', error);
    }
  },
  
  updateInstance: (instanceId: string, instance: WorkflowInstance) => {
    set((state) => {
      const newInstances = new Map(state.instances);
      newInstances.set(instanceId, instance);
      return { instances: newInstances };
    });
  },
  
  setError: (error: string | null) => {
    set({ error });
  },
}));