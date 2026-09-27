import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query'
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

export interface RenderResponse {
    success: boolean
    rendered_path: string | null
    duration_ms: number | null
    error: string | null
}

export interface RenderAllResponse {
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

export const BUMPERS_QUERY_KEY = ['bumpers']
export const BUMPER_BACKS_QUERY_KEY = ['bumper-backs']

const fetchBumpers = async (): Promise<Bumper[]> => {
    const response = await apiClient.get('/api/bumpers')
    return response.data
}

export const useBumpers = () =>
    useQuery({ queryKey: BUMPERS_QUERY_KEY, queryFn: fetchBumpers })

const fetchBumperBacks = async (): Promise<BumperBack[]> => {
    const response = await apiClient.get('/api/bumper-backs')
    return response.data
}

export const useBumperBacks = () =>
    useQuery({ queryKey: BUMPER_BACKS_QUERY_KEY, queryFn: fetchBumperBacks })

const invalidateBumpers = (queryClient: QueryClient) =>
    queryClient.invalidateQueries({ queryKey: BUMPERS_QUERY_KEY })

const invalidateBumperBacks = (queryClient: QueryClient) =>
    queryClient.invalidateQueries({ queryKey: BUMPER_BACKS_QUERY_KEY })

export const useCreateBumper = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (data: Partial<Bumper>) => {
            const response = await apiClient.post('/api/bumpers', data)
            return response.data as Bumper
        },
        onSuccess: () => invalidateBumpers(queryClient),
    })
}

export const useUpdateBumper = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ id, data }: { id: number; data: Partial<Bumper> }) => {
            const response = await apiClient.put(`/api/bumpers/${id}`, data)
            return response.data as Bumper
        },
        onSuccess: () => invalidateBumpers(queryClient),
    })
}

export const useDeleteBumper = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            await apiClient.delete(`/api/bumpers/${id}`)
        },
        onSuccess: () => invalidateBumpers(queryClient),
    })
}

export const useRenderBumper = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            const response = await apiClient.post(`/api/bumpers/${id}/render`)
            return response.data as RenderResponse
        },
        onSuccess: () => invalidateBumpers(queryClient),
    })
}

export const useRenderAllBumpers = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async () => {
            const response = await apiClient.post('/api/bumpers/render-all')
            return response.data as RenderAllResponse
        },
        onSuccess: () => invalidateBumpers(queryClient),
    })
}

export const useDownloadBumperBack = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ url, name }: { url: string; name: string }) => {
            const response = await apiClient.post('/api/bumper-backs/fetch', { url, name })
            return response.data as BumperBack
        },
        onSuccess: () => invalidateBumperBacks(queryClient),
    })
}

export const useUploadBumperBack = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ file, name }: { file: File; name: string }) => {
            const formData = new FormData()
            formData.append('file', file)
            formData.append('name', name)
            const response = await apiClient.post('/api/bumper-backs/upload', formData, {
                headers: { 'Content-Type': 'multipart/form-data' },
            })
            return response.data as BumperBack
        },
        onSuccess: () => invalidateBumperBacks(queryClient),
    })
}

export const useDeleteBumperBack = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            await apiClient.delete(`/api/bumper-backs/${id}`)
        },
        onSuccess: () => invalidateBumperBacks(queryClient),
    })
}

export const useRenderBumperBack = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            const response = await apiClient.post(`/api/bumper-backs/${id}/render`)
            return response.data as RenderResponse
        },
        onSuccess: () => invalidateBumperBacks(queryClient),
    })
}

export const useRenderAllBumperBacks = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async () => {
            const response = await apiClient.post('/api/bumper-backs/render-all')
            return response.data as RenderAllResponse
        },
        onSuccess: () => invalidateBumperBacks(queryClient),
    })
}
