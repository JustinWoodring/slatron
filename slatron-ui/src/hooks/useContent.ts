import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
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

export type NewContentItem = Omit<ContentItem, 'id' | 'created_at'>

export const CONTENT_QUERY_KEY = ['content']

const fetchContent = async (): Promise<ContentItem[]> => {
    const response = await apiClient.get('/api/content')
    return response.data
}

export const useContent = () =>
    useQuery({ queryKey: CONTENT_QUERY_KEY, queryFn: fetchContent })

export const useCreateContent = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (newContent: NewContentItem) => {
            const response = await apiClient.post('/api/content', newContent)
            return response.data as ContentItem
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: CONTENT_QUERY_KEY }),
    })
}

export const useUpdateContent = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ id, updates }: { id: number; updates: Partial<ContentItem> }) => {
            const response = await apiClient.put(`/api/content/${id}`, updates)
            return response.data as ContentItem
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: CONTENT_QUERY_KEY }),
    })
}

export const useDeleteContent = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            await apiClient.delete(`/api/content/${id}`)
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: CONTENT_QUERY_KEY }),
    })
}
