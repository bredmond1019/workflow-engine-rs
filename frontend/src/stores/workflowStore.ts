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
  pollingInterval: number | null;
  
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
  startPolling: () => void;
  stopPolling: () => void;
}


export const workflowStore = create<WorkflowState>((set, get) => ({
  templates: [],
  instances: new Map<string, WorkflowInstance>(),
  availableWorkflows: [],
  categories: [],
  tags: [],
  isLoading: false,
  error: null,
  pollingInterval: null,
  
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
    } catch (error: any) {
      set({
        error: error.response?.data?.message || 'Failed to fetch available workflows',
      });
      throw error;
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
      
      // Create initial instance from response
      const instance: WorkflowInstance = {
        instance_id,
        workflow_name: instanceData.workflow_name || workflowName,
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
        error: error.response?.data?.message || 'Failed to trigger workflow',
      });
      throw error;
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
  
  startPolling: () => {
    const { pollingInterval } = get();
    if (pollingInterval) return; // Already polling
    
    const interval = setInterval(async () => {
      const { instances } = get();
      const runningInstances = Array.from(instances.values()).filter(
        instance => instance.status === 'Running' || instance.status === 'Created'
      );
      
      // Poll status for all running workflows
      for (const instance of runningInstances) {
        try {
          await get().fetchWorkflowStatus(instance.instance_id);
        } catch (error) {
          console.error(`Failed to poll status for ${instance.instance_id}:`, error);
        }
      }
    }, 2000); // Poll every 2 seconds
    
    set({ pollingInterval: interval as unknown as number });
  },
  
  stopPolling: () => {
    const { pollingInterval } = get();
    if (pollingInterval) {
      clearInterval(pollingInterval);
      set({ pollingInterval: null });
    }
  },
}));