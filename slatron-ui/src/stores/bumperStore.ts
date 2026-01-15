import { create } from 'zustand'
import { apiClient } from '../api/client'

export interface Bumper {
    id: number
    name: string
    bumper_type: 'station_ident' | 'transition' | 'show_opener' | 'lower_third' | 'custom'
    description: string | null
    is_template: boolean
    template_content: string | null
    rendered_path: string | null
    duration_ms: number | null
    is_builtin: boolean
    created_at: string
    updated_at: string
    last_rendered_at: string | null
    bumper_back_id: number | null
}

interface RenderResponse {
    success: boolean
    rendered_path: string | null
    duration_ms: number | null
    error: string | null
}

interface RenderAllResponse {
    total: number
    successful: number
    failed: number
    errors: string[]
}

export interface BumperBack {
    id: number
    name: string
    description: string | null
    file_path: string
    duration_ms: number | null
    is_builtin: boolean
    created_at: string
    updated_at: string
}

interface BumperStore {
    bumpers: Bumper[]
    bumperBacks: BumperBack[]
    loading: boolean
    error: string | null
    fetchBumpers: () => Promise<void>
    createBumper: (data: Partial<Bumper>) => Promise<Bumper>
    updateBumper: (id: number, data: Partial<Bumper>) => Promise<Bumper>
    deleteBumper: (id: number) => Promise<void>
    renderBumper: (id: number) => Promise<RenderResponse>
    renderAllBumpers: () => Promise<RenderAllResponse>

    // Bumper Back methods
    fetchBumperBacks: () => Promise<void>
    downloadBumperBack: (url: string, name: string) => Promise<BumperBack>
    uploadBumperBack: (file: File, name: string) => Promise<BumperBack>
    deleteBumperBack: (id: number) => Promise<void>
    renderBumperBack: (id: number) => Promise<RenderResponse>
    renderAllBumperBacks: () => Promise<RenderAllResponse>
}

export const useBumperStore = create<BumperStore>((set, get) => ({
    bumpers: [],
    bumperBacks: [],
    loading: false,
    error: null,

    fetchBumpers: async () => {
        try {
            set({ loading: true, error: null })
            const response = await apiClient.get('/api/bumpers')
            set({ bumpers: response.data, loading: false })
        } catch (error: any) {
            set({ error: error.message, loading: false })
            throw error
        }
    },

    createBumper: async (data) => {
        const response = await apiClient.post('/api/bumpers', data)
        set({ bumpers: [...get().bumpers, response.data] })
        return response.data
    },

    updateBumper: async (id, data) => {
        const response = await apiClient.put(`/api/bumpers/${id}`, data)
        set({
            bumpers: get().bumpers.map((b) =>
                b.id === id ? response.data : b
            ),
        })
        return response.data
    },

    deleteBumper: async (id) => {
        await apiClient.delete(`/api/bumpers/${id}`)
        set({
            bumpers: get().bumpers.filter((b) => b.id !== id),
        })
    },

    renderBumper: async (id) => {
        const response = await apiClient.post(`/api/bumpers/${id}/render`)
        // Refresh bumpers to get updated rendered_path and duration
        await get().fetchBumpers()
        return response.data
    },

    renderAllBumpers: async () => {
        const response = await apiClient.post('/api/bumpers/render-all')
        // Refresh bumpers to get all updates
        await get().fetchBumpers()
        return response.data
    },

    // Bumper Back methods
    fetchBumperBacks: async () => {
        try {
            // Don't set loading global here to avoid flickering entire UI for sub-fetches
            const response = await apiClient.get('/api/bumper-backs')
            set({ bumperBacks: response.data })
        } catch (error: any) {
            console.error("Failed to fetch bumper backs", error)
        }
    },

    downloadBumperBack: async (url, name) => {
        set({ loading: true })
        try {
            const response = await apiClient.post('/api/bumper-backs/fetch', { url, name })
            set({
                bumperBacks: [...get().bumperBacks, response.data],
                loading: false
            })
            return response.data
        } catch (error: any) {
            set({ loading: false, error: error.message })
            throw error
        }
    },

    uploadBumperBack: async (file, name) => {
        set({ loading: true })
        try {
            const formData = new FormData()
            formData.append('file', file)
            formData.append('name', name)

            const response = await apiClient.post('/api/bumper-backs/upload', formData, {
                headers: {
                    'Content-Type': 'multipart/form-data'
                }
            })

            set({
                bumperBacks: [...get().bumperBacks, response.data],
                loading: false
            })
            return response.data
        } catch (error: any) {
            set({ loading: false, error: error.message })
            throw error
        }
    },

    deleteBumperBack: async (id) => {
        await apiClient.delete(`/api/bumper-backs/${id}`)
        set({
            bumperBacks: get().bumperBacks.filter((b) => b.id !== id),
        })
    },

    renderBumperBack: async (id) => {
        const response = await apiClient.post(`/api/bumper-backs/${id}/render`)
        await get().fetchBumperBacks()
        return response.data
    },

    renderAllBumperBacks: async () => {
        const response = await apiClient.post('/api/bumper-backs/render-all')
        await get().fetchBumperBacks()
        return response.data
    },
}))
