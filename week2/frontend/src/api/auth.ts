import { api } from '@/api/client'
import type {
  LoginRequest,
  LoginResponse,
  User,
} from '@/types/api'

export const authApi = {
  login: async (data: LoginRequest): Promise<LoginResponse> => {
    const response = await api.post<LoginResponse>('/auth/login', data)
    return response.data
  },

  register: async (data: {
    username: string
    email: string
    password: string
  }): Promise<User> => {
    const response = await api.post<User>('/auth/register', data)
    return response.data
  },

  refresh: async (refreshToken: string): Promise<LoginResponse> => {
    const response = await api.post<LoginResponse>('/auth/refresh', {
      refresh_token: refreshToken,
    })
    return response.data
  },

  logout: async (): Promise<void> => {
    await api.post('/auth/logout')
  },
}
