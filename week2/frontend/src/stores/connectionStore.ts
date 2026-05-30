import { create } from 'zustand'
import type { DatabaseConnection } from '@/types/api'

interface ConnectionState {
  connections: DatabaseConnection[]
  currentConnectionId: string | null

  setConnections: (connections: DatabaseConnection[]) => void
  setCurrentConnection: (id: string | null) => void
  addConnection: (connection: DatabaseConnection) => void
  updateConnection: (id: string, connection: Partial<DatabaseConnection>) => void
  removeConnection: (id: string) => void
}

export const useConnectionStore = create<ConnectionState>()((set) => ({
  connections: [],
  currentConnectionId: null,

  setConnections: (connections) => set({ connections }),

  setCurrentConnection: (id) => set({ currentConnectionId: id }),

  addConnection: (connection) =>
    set((state) => ({
      connections: [...state.connections, connection],
    })),

  updateConnection: (id, updates) =>
    set((state) => ({
      connections: state.connections.map((conn) =>
        conn.id === id ? { ...conn, ...updates } : conn
      ),
    })),

  removeConnection: (id) =>
    set((state) => ({
      connections: state.connections.filter((conn) => conn.id !== id),
      currentConnectionId:
        state.currentConnectionId === id ? null : state.currentConnectionId,
    })),
}))
