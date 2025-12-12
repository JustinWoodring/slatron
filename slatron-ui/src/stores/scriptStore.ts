import { create } from 'zustand'
import { apiClient } from '../api/client'

export interface Script {
    id: number
    name: string
    description: string | null
    script_type: 'content_loader' | 'overlay' | 'global' | 'adapter' | 'content' | 'utility' | string
    script_content: string
    parameters_schema: string | null
    is_builtin: boolean
    created_at: string
    updated_at: string
}

interface ValidateResponse {
    valid: boolean
    errors: string[]
}

interface ExecuteResponse {
    success: boolean
    result?: string
    mpv_commands?: string[]
    error?: string
}

interface ScriptStore {
    scripts: Script[]
    fetchScripts: () => Promise<void>
    createScript: (data: Partial<Script>) => Promise<Script>
    updateScript: (id: number, data: Partial<Script>) => Promise<Script>
    deleteScript: (id: number) => Promise<void>
    validateScript: (content: string, type: string) => Promise<ValidateResponse>
    executeScript: (id: number, params: any) => Promise<ExecuteResponse>
}

export const useScriptStore = create<ScriptStore>((set, get) => ({
    scripts: [],

    fetchScripts: async () => {
        const response = await apiClient.get('/api/scripts')
        set({ scripts: response.data })
    },

    createScript: async (data) => {
        const response = await apiClient.post('/api/scripts', data)
        set({ scripts: [...get().scripts, response.data] })
        return response.data
    },

    updateScript: async (id, data) => {
        const response = await apiClient.put(`/api/scripts/${id}`, data)
        set({
            scripts: get().scripts.map((s) =>
                s.id === id ? response.data : s
            ),
        })
        return response.data
    },

    deleteScript: async (id) => {
        await apiClient.delete(`/api/scripts/${id}`)
        set({
            scripts: get().scripts.filter((s) => s.id !== id),
        })
    },

    validateScript: async (content, type) => {
        // Use ID 0 for validation request pattern
        const response = await apiClient.post(`/api/scripts/0/validate`, {
            script_content: content,
            script_type: type
        })
        return response.data
    },

    executeScript: async (id, params) => {
        const response = await apiClient.post(`/api/scripts/${id}/execute`, { params })
        return response.data
    }
}))
