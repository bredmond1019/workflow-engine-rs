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
          set({
            isLoading: false,
            error: error.response?.data?.message || 'Authentication failed',
          });
          throw error;
        }
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
          const response = await apiClient.post(API_ENDPOINTS.auth.verify, {
            token,
          });
          
          if (response.data.valid && response.data.claims) {
            // Update user info from verified claims
            const claims = response.data.claims;
            const user: User = {
              sub: claims.sub,
              role: claims.role,
              exp: claims.exp,
              iat: claims.iat,
            };
            
            set({ user, isAuthenticated: true });
            return true;
          } else {
            set({ token: null, user: null, isAuthenticated: false });
            return false;
          }
        } catch {
          set({ token: null, user: null, isAuthenticated: false });
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