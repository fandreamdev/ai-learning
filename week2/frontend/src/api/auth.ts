import { api } from '@/api/client'
import { useAuthStore } from '@/stores/authStore'
import type {
  ApiResponse,
  LoginRequest,
  LoginResponse,
  User,
} from '@/types/api'

export const authApi = {
  login: async (data: LoginRequest): Promise<LoginResponse> => {
    const response = await api.post<ApiResponse<LoginResponse>>('/auth/login', data)
    if (!response.data.data) {
      throw new Error(response.data.message || '登录失败')
    }
    return response.data.data
  },

  register: async (data: {
    username: string
    email: string
    password: string
  }): Promise<User> => {
    const response = await api.post<ApiResponse<User>>('/auth/register', data)
    if (!response.data.data) {
      throw new Error(response.data.message || '注册失败')
    }
    return response.data.data
  },

  refresh: async (refreshToken: string): Promise<LoginResponse> => {
    const response = await api.post<ApiResponse<LoginResponse>>('/auth/refresh', {
      refresh_token: refreshToken,
    })
    if (!response.data.data) {
      throw new Error(response.data.message || '刷新 Token 失败')
    }
    return response.data.data
  },

  logout: async (): Promise<void> => {
    const { accessToken, refreshToken } = useAuthStore.getState()
    await api.post('/auth/logout', {
      access_token: accessToken,
      refresh_token: refreshToken,
    })
  },
}
