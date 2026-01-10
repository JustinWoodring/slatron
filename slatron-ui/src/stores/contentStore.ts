import { create } from 'zustand'
import { apiClient } from '../api/client'

export interface ContentItem {
    id: number
    title: string
    description: string | null
    content_type: string
    content_path: string
    duration_minutes: number | null
    tags: string | null
    node_accessibility: string | null
    created_at: string
    transformer_scripts: string | null
    is_dj_accessible: boolean
}

interface ContentStore {
    content: ContentItem[]
    fetchContent: () => Promise<void>
    createContent: (content: Omit<ContentItem, 'id' | 'created_at'>) => Promise<void>
    updateContent: (id: number, content: Partial<ContentItem>) => Promise<void>
    deleteContent: (id: number) => Promise<void>
}

export const useContentStore = create<ContentStore>((set) => ({
    content: [],
    fetchContent: async () => {
        try {
            const response = await apiClient.get('/api/content')
            set({ content: response.data })
        } catch (error) {
            console.error('Failed to fetch content:', error)
        }
    },
    createContent: async (newContent) => {
        try {
            const response = await apiClient.post('/api/content', newContent)
            set((state) => ({ content: [...state.content, response.data] }))
        } catch (error) {
            console.error('Failed to create content:', error)
            throw error
        }
    },
    updateContent: async (id, updates) => {
        try {
            const response = await apiClient.put(`/api/content/${id}`, updates)
            set((state) => ({
                content: state.content.map(c => c.id === id ? response.data : c)
            }))
        } catch (error) {
            console.error('Failed to update content:', error)
            throw error
        }
    },
    deleteContent: async (id) => {
        try {
            await apiClient.delete(`/api/content/${id}`)
            set((state) => ({
                content: state.content.filter(c => c.id !== id)
            }))
        } catch (error) {
            console.error('Failed to delete content:', error)
            throw error
        }
    }
}))
