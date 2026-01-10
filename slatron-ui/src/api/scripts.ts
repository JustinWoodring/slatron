import { apiClient } from './client'

export interface Script {
    id: number
    name: string
    description: string | null
    script_type: string
    script_content: string
    parameters_schema: string | null
    is_builtin: boolean
    created_at: string
    updated_at: string
}

export const getScripts = () => apiClient.get<Script[]>('/api/scripts').then(res => res.data)
export const getScript = (id: number) => apiClient.get<Script>(`/api/scripts/${id}`).then(res => res.data)
export const createScript = (data: Partial<Script>) => apiClient.post<Script>('/api/scripts', data).then(res => res.data)
export const updateScript = (id: number, data: Partial<Script>) => apiClient.put<Script>(`/api/scripts/${id}`, data).then(res => res.data)
export const deleteScript = (id: number) => apiClient.delete(`/api/scripts/${id}`)
