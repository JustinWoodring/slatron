import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { getScripts, createScript, updateScript, deleteScript, Script } from '../api/scripts'
import { apiClient } from '../api/client'

export type { Script }

export interface ValidateResponse {
    valid: boolean
    errors: string[]
}

export interface ExecuteResponse {
    success: boolean
    result?: string
    mpv_commands?: string[]
    error?: string
}

export const SCRIPTS_QUERY_KEY = ['scripts']

const fetchScripts = (): Promise<Script[]> => getScripts()

export const useScripts = () =>
    useQuery({ queryKey: SCRIPTS_QUERY_KEY, queryFn: fetchScripts })

export const useCreateScript = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: (data: Partial<Script>) => createScript(data),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: SCRIPTS_QUERY_KEY }),
    })
}

export const useUpdateScript = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: ({ id, data }: { id: number; data: Partial<Script> }) => updateScript(id, data),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: SCRIPTS_QUERY_KEY }),
    })
}

export const useDeleteScript = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: (id: number) => deleteScript(id),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: SCRIPTS_QUERY_KEY }),
    })
}

// One-shot operations: no server-cache state to invalidate.
export const validateScript = async (content: string, type: string): Promise<ValidateResponse> => {
    // Use ID 0 for validation request pattern
    const response = await apiClient.post('/api/scripts/0/validate', {
        script_content: content,
        script_type: type,
    })
    return response.data
}

export const executeScript = async (id: number, params: Record<string, unknown>): Promise<ExecuteResponse> => {
    const response = await apiClient.post(`/api/scripts/${id}/execute`, { params })
    return response.data
}
