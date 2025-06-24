import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { AuthToken, User } from '../types';
import { apiClient, API_ENDPOINTS } from '../api/config';

interface AuthState {
  token: string | null;
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  login: (sub: string, role: string) => Promise<void>;
  loginDemo: (sub: string, role: string) => void;
  logout: () => void;
  verifyToken: () => Promise<boolean>;
  setError: (error: string | null) => void;
}

export const authStore = create<AuthState>()(
  persist(
    (set, get) => ({
      token: null,
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
      
      login: async (sub: string, role: string) => {
        set({ isLoading: true, error: null });
        try {
          const response = await apiClient.post<AuthToken>(API_ENDPOINTS.auth.token, {
            sub,
            role,
          });
          
          const { access_token } = response.data;
          
          // Decode token to get user info (simple base64 decode for JWT)
          const tokenParts = access_token.split('.');
          if (tokenParts.length === 3) {
            const payload = JSON.parse(atob(tokenParts[1]));
            const user: User = {
              sub: payload.sub,
              role: payload.role,
              exp: payload.exp,
              iat: payload.iat,
            };
            
            set({
              token: access_token,
              user,
              isAuthenticated: true,
              isLoading: false,
              error: null,
            });
          }
        } catch (error: any) {
          // Fallback to demo mode if API is not available
          console.warn('API not available, using demo mode');
          get().loginDemo(sub, role);
        }
      },
      
      loginDemo: (sub: string, role: string) => {
        // Create a mock JWT token for demo purposes
        const mockPayload = {
          sub,
          role,
          exp: Math.floor(Date.now() / 1000) + 86400, // 24 hours from now
          iat: Math.floor(Date.now() / 1000)
        };
        
        const mockToken = `demo.${btoa(JSON.stringify(mockPayload))}.signature`;
        
        const user: User = {
          sub,
          role,
          exp: mockPayload.exp,
          iat: mockPayload.iat,
        };
        
        set({
          token: mockToken,
          user,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });
      },
      
      logout: () => {
        set({
          token: null,
          user: null,
          isAuthenticated: false,
          error: null,
        });
      },
      
      verifyToken: async () => {
        const token = get().token;
        if (!token) return false;
        
        try {
          const response = await apiClient.get(API_ENDPOINTS.auth.verify, {
            params: { token },
          });
          
          return response.data.valid;
        } catch {
          return false;
        }
      },
      
      setError: (error: string | null) => {
        set({ error });
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        token: state.token,
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);