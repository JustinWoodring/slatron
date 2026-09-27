import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query'
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

export interface NewSpotReelItem {
    item_type: string
    item_path: string
    display_duration_secs?: number
    title?: string
}

export const SPOT_REELS_QUERY_KEY = ['spot-reels']
export const spotReelQueryKey = (id: number) => ['spot-reels', id]

const fetchSpotReels = async (): Promise<SpotReelListEntry[]> => {
    const response = await apiClient.get('/api/spot-reels')
    return response.data
}

export const useSpotReels = () =>
    useQuery({ queryKey: SPOT_REELS_QUERY_KEY, queryFn: fetchSpotReels })

const fetchSpotReel = async (id: number): Promise<SpotReelWithItems> => {
    const response = await apiClient.get(`/api/spot-reels/${id}`)
    return response.data
}

export const useSpotReel = (id: number | null) =>
    useQuery({
        queryKey: spotReelQueryKey(id ?? 0),
        queryFn: () => fetchSpotReel(id as number),
        enabled: id !== null,
    })

const invalidateReels = (queryClient: QueryClient) =>
    queryClient.invalidateQueries({ queryKey: SPOT_REELS_QUERY_KEY })

const invalidateReel = (queryClient: QueryClient, reelId: number) =>
    queryClient.invalidateQueries({ queryKey: spotReelQueryKey(reelId) })

export const useCreateSpotReel = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ title, description }: { title: string; description?: string | null }) => {
            const response = await apiClient.post('/api/spot-reels', { title, description })
            return response.data as SpotReel
        },
        onSuccess: () => invalidateReels(queryClient),
    })
}

export const useUpdateSpotReel = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ id, data }: { id: number; data: Partial<SpotReel> }) => {
            const response = await apiClient.put(`/api/spot-reels/${id}`, data)
            return response.data as SpotReel
        },
        onSuccess: (_updated, variables) => {
            invalidateReels(queryClient)
            invalidateReel(queryClient, variables.id)
        },
    })
}

export const useDeleteSpotReel = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            await apiClient.delete(`/api/spot-reels/${id}`)
        },
        onSuccess: () => invalidateReels(queryClient),
    })
}

export const useAddSpotReelItem = (reelId: number) => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (data: NewSpotReelItem) => {
            const response = await apiClient.post(`/api/spot-reels/${reelId}/items`, data)
            return response.data as SpotReelItem
        },
        onSuccess: () => invalidateReel(queryClient, reelId),
    })
}

export const useUpdateSpotReelItem = (reelId: number) => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ itemId, data }: { itemId: number; data: Partial<SpotReelItem> }) => {
            const response = await apiClient.put(`/api/spot-reels/${reelId}/items/${itemId}`, data)
            return response.data as SpotReelItem
        },
        onSuccess: () => invalidateReel(queryClient, reelId),
    })
}

export const useDeleteSpotReelItem = (reelId: number) => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (itemId: number) => {
            await apiClient.delete(`/api/spot-reels/${reelId}/items/${itemId}`)
        },
        onSuccess: () => invalidateReel(queryClient, reelId),
    })
}

export const useReorderSpotReelItems = (reelId: number) => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (items: { id: number; position: number }[]) => {
            const response = await apiClient.put(`/api/spot-reels/${reelId}/items/reorder`, { items })
            return response.data as SpotReelItem[]
        },
        onSuccess: () => invalidateReel(queryClient, reelId),
    })
}
