import { create } from 'zustand'
import { apiClient } from '../api/client'

export interface SpotReelItem {
    id: number
    spot_reel_id: number
    item_type: 'image' | 'video' | 'web'
    item_path: string
    display_duration_secs: number
    position: number
    title: string | null
    created_at: string
    updated_at: string
}

export interface SpotReel {
    id: number
    title: string
    description: string | null
    created_at: string
    updated_at: string
}

export interface SpotReelWithItems extends SpotReel {
    items: SpotReelItem[]
    content_item_id: number | null
}

export interface SpotReelListEntry extends SpotReel {
    item_count: number
    total_duration_secs: number
    content_item_id: number | null
}

interface SpotReelStore {
    reels: SpotReelListEntry[]
    currentReel: SpotReelWithItems | null
    loading: boolean
    error: string | null

    fetchSpotReels: () => Promise<void>
    fetchSpotReel: (id: number) => Promise<void>
    createSpotReel: (title: string, description?: string) => Promise<SpotReelWithItems>
    updateSpotReel: (id: number, data: { title?: string; description?: string }) => Promise<void>
    deleteSpotReel: (id: number) => Promise<void>

    addItem: (reelId: number, data: { item_type: string; item_path: string; display_duration_secs?: number; title?: string }) => Promise<SpotReelItem>
    updateItem: (reelId: number, itemId: number, data: Partial<SpotReelItem>) => Promise<SpotReelItem>
    deleteItem: (reelId: number, itemId: number) => Promise<void>
    reorderItems: (reelId: number, items: { id: number; position: number }[]) => Promise<void>
}

export const useSpotReelStore = create<SpotReelStore>((set, get) => ({
    reels: [],
    currentReel: null,
    loading: false,
    error: null,

    fetchSpotReels: async () => {
        try {
            set({ loading: true, error: null })
            const response = await apiClient.get('/api/spot-reels')
            set({ reels: response.data, loading: false })
        } catch (error: any) {
            set({ error: error.message, loading: false })
            throw error
        }
    },

    fetchSpotReel: async (id) => {
        try {
            set({ loading: true, error: null })
            const response = await apiClient.get(`/api/spot-reels/${id}`)
            set({ currentReel: response.data, loading: false })
        } catch (error: any) {
            set({ error: error.message, loading: false })
            throw error
        }
    },

    createSpotReel: async (title, description) => {
        const response = await apiClient.post('/api/spot-reels', { title, description })
        set({ reels: [...get().reels, { ...response.data, item_count: 0, total_duration_secs: 0 }] })
        return response.data
    },

    updateSpotReel: async (id, data) => {
        const response = await apiClient.put(`/api/spot-reels/${id}`, data)
        set({
            reels: get().reels.map((r) => r.id === id ? { ...r, ...response.data } : r),
        })
        // Also update currentReel if it's the same
        const current = get().currentReel
        if (current && current.id === id) {
            set({ currentReel: { ...current, ...response.data } })
        }
    },

    deleteSpotReel: async (id) => {
        await apiClient.delete(`/api/spot-reels/${id}`)
        set({ reels: get().reels.filter((r) => r.id !== id) })
        if (get().currentReel?.id === id) {
            set({ currentReel: null })
        }
    },

    addItem: async (reelId, data) => {
        const response = await apiClient.post(`/api/spot-reels/${reelId}/items`, data)
        const current = get().currentReel
        if (current && current.id === reelId) {
            set({ currentReel: { ...current, items: [...current.items, response.data] } })
        }
        return response.data
    },

    updateItem: async (reelId, itemId, data) => {
        const response = await apiClient.put(`/api/spot-reels/${reelId}/items/${itemId}`, data)
        const current = get().currentReel
        if (current && current.id === reelId) {
            set({
                currentReel: {
                    ...current,
                    items: current.items.map((i) => i.id === itemId ? response.data : i),
                },
            })
        }
        return response.data
    },

    deleteItem: async (reelId, itemId) => {
        await apiClient.delete(`/api/spot-reels/${reelId}/items/${itemId}`)
        const current = get().currentReel
        if (current && current.id === reelId) {
            set({
                currentReel: {
                    ...current,
                    items: current.items.filter((i) => i.id !== itemId),
                },
            })
        }
    },

    reorderItems: async (reelId, items) => {
        const response = await apiClient.put(`/api/spot-reels/${reelId}/items/reorder`, { items })
        const current = get().currentReel
        if (current && current.id === reelId) {
            set({ currentReel: { ...current, items: response.data } })
        }
    },
}))
