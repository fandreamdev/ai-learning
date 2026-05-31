import { create } from 'zustand'
import type { Message, Conversation } from '@/types/api'

interface ChatState {
  conversations: Conversation[]
  currentConversationId: string | null
  messages: Message[]
  isLoading: boolean

  setConversations: (conversations: Conversation[]) => void
  setCurrentConversation: (id: string | null) => void
  setMessages: (messages: Message[]) => void
  addMessage: (message: Message) => void
  updateLastMessage: (message: Partial<Message>) => void
  setLoading: (loading: boolean) => void
  clearMessages: () => void
}

export const useChatStore = create<ChatState>()((set) => ({
  conversations: [],
  currentConversationId: null,
  messages: [],
  isLoading: false,

  setConversations: (conversations) => set({ conversations }),

  setCurrentConversation: (id) =>
    set({
      currentConversationId: id,
      messages: [],
    }),

  setMessages: (messages) => set({ messages }),

  addMessage: (message) =>
    set((state) => ({
      messages: [...state.messages, message],
    })),

  updateLastMessage: (updates) =>
    set((state) => ({
      messages: state.messages.map((msg, i) =>
        i === state.messages.length - 1 ? { ...msg, ...updates } : msg
      ),
    })),

  setLoading: (loading) => set({ isLoading: loading }),

  clearMessages: () => set({ messages: [] }),
}))
