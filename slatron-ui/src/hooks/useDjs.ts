import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query'
import {
    getDjs, createDj, deleteDj, updateDj, DjProfile, NewDjProfile,
    getAiProviders, createAiProvider, updateAiProvider, deleteAiProvider, AiProvider, NewAiProvider
} from '../api/dj'

export const DJS_QUERY_KEY = ['djs']
export const AI_PROVIDERS_QUERY_KEY = ['ai-providers']

const fetchDjs = async (): Promise<DjProfile[]> => getDjs()

export const useDjs = () =>
    useQuery({ queryKey: DJS_QUERY_KEY, queryFn: fetchDjs })

const invalidateDjs = (queryClient: QueryClient) =>
    queryClient.invalidateQueries({ queryKey: DJS_QUERY_KEY })

export const useCreateDj = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: (data: NewDjProfile) => createDj(data),
        onSuccess: () => invalidateDjs(queryClient),
    })
}

export const useUpdateDj = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: ({ id, data }: { id: number; data: Partial<NewDjProfile> }) => updateDj(id, data),
        onSuccess: () => invalidateDjs(queryClient),
    })
}

export const useDeleteDj = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: (id: number) => deleteDj(id),
        onSuccess: () => invalidateDjs(queryClient),
    })
}

const fetchAiProviders = async (): Promise<AiProvider[]> => getAiProviders()

export const useAiProviders = () =>
    useQuery({ queryKey: AI_PROVIDERS_QUERY_KEY, queryFn: fetchAiProviders })

const invalidateAiProviders = (queryClient: QueryClient) =>
    queryClient.invalidateQueries({ queryKey: AI_PROVIDERS_QUERY_KEY })

export const useCreateAiProvider = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: (data: NewAiProvider) => createAiProvider(data),
        onSuccess: () => invalidateAiProviders(queryClient),
    })
}

export const useUpdateAiProvider = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: ({ id, data }: { id: number; data: Partial<AiProvider> }) => updateAiProvider(id, data),
        onSuccess: () => invalidateAiProviders(queryClient),
    })
}

export const useDeleteAiProvider = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: (id: number) => deleteAiProvider(id),
        onSuccess: () => invalidateAiProviders(queryClient),
    })
}
