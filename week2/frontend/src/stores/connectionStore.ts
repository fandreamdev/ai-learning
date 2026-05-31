import { create } from 'zustand'
import type { DatabaseConnection } from '@/types/api'
import type { CreateConnectionRequest, UpdateConnectionRequest } from '@/types/api'
import { api } from '@/api/client'

interface ConnectionState {
  connections: DatabaseConnection[]
  currentConnectionId: string | null
  selectedConnectionId: string | null
  loading: boolean

  setConnections: (connections: DatabaseConnection[]) => void
  setCurrentConnection: (id: string | null) => void
  addConnection: (connection: DatabaseConnection) => void
  updateConnection: (id: string, request: UpdateConnectionRequest) => Promise<void>
  removeConnection: (id: string) => void
  fetchConnections: () => Promise<void>
  createConnection: (request: CreateConnectionRequest) => Promise<void>
  deleteConnection: (id: string) => Promise<void>
  testConnection: (id: string) => Promise<{ success: boolean; message?: string }>
  setDefaultConnection: (id: string) => Promise<void>
}

export const useConnectionStore = create<ConnectionState>()((set) => ({
  connections: [],
  currentConnectionId: null,
  selectedConnectionId: null,
  loading: false,

  setConnections: (connections) => set({ connections }),

  setCurrentConnection: (id) => set({ currentConnectionId: id, selectedConnectionId: id }),

  addConnection: (connection) =>
    set((state) => ({
      connections: [...state.connections, connection],
    })),

  updateConnection: async (id, request) => {
    const response = await api.put(`/connections/${id}`, request)
    const connection = response.data?.data
    if (connection) {
      set((state) => ({
        connections: state.connections.map((conn) =>
          conn.id === id ? connection : conn
        ),
      }))
    }
  },

  removeConnection: (id) =>
    set((state) => ({
      connections: state.connections.filter((conn) => conn.id !== id),
      currentConnectionId:
        state.currentConnectionId === id ? null : state.currentConnectionId,
      selectedConnectionId:
        state.selectedConnectionId === id ? null : state.selectedConnectionId,
    })),

  fetchConnections: async () => {
    set({ loading: true })
    try {
      const response = await api.get('/connections')
      const items = response.data?.data?.items ?? []
      set({ connections: items, loading: false })
    } catch {
      set({ loading: false })
      throw new Error('Failed to fetch connections')
    }
  },

  createConnection: async (request) => {
    const response = await api.post('/connections', request)
    const connection = response.data?.data
    if (connection) {
      set((state) => ({ connections: [...state.connections, connection] }))
    }
  },

  deleteConnection: async (id) => {
    await api.delete(`/connections/${id}`)
    set((state) => ({
      connections: state.connections.filter((conn) => conn.id !== id),
      currentConnectionId: state.currentConnectionId === id ? null : state.currentConnectionId,
      selectedConnectionId: state.selectedConnectionId === id ? null : state.selectedConnectionId,
    }))
  },

  testConnection: async (id) => {
    const response = await api.post(`/connections/${id}/test`)
    return response.data?.data ?? { success: false, message: response.data?.message }
  },

  setDefaultConnection: async (id) => {
    await api.put(`/connections/${id}/default`)
    set((state) => ({
      connections: state.connections.map((conn) => ({
        ...conn,
        is_default: conn.id === id,
      })),
    }))
  },
}))
