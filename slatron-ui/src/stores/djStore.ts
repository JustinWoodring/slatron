import { create } from 'zustand'
import {
    getDjs, createDj, deleteDj, updateDj, DjProfile, NewDjProfile,
    getAiProviders, createAiProvider, updateAiProvider, deleteAiProvider, AiProvider, NewAiProvider
} from '../api/dj'

interface DjStore {
    djs: DjProfile[]
    aiProviders: AiProvider[]
    isLoading: boolean
    error: string | null

    fetchDjs: () => Promise<void>
    addDj: (data: NewDjProfile) => Promise<void>
    updateDj: (id: number, data: Partial<NewDjProfile>) => Promise<void>
    removeDj: (id: number) => Promise<void>

    fetchAiProviders: () => Promise<void>
    addAiProvider: (data: NewAiProvider) => Promise<void>
    removeAiProvider: (id: number) => Promise<void>
    updateAiProvider: (id: number, data: Partial<AiProvider>) => Promise<void>
}

export const useDjStore = create<DjStore>((set, get) => ({
    djs: [],
    aiProviders: [],
    isLoading: false,
    error: null,

    fetchDjs: async () => {
        set({ isLoading: true, error: null })
        try {
            const djs = await getDjs()
            set({ djs, isLoading: false })
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
        }
    },

    addDj: async (data) => {
        set({ isLoading: true, error: null })
        try {
            await createDj(data)
            await get().fetchDjs()
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
            throw error
        }
    },

    updateDj: async (id, data) => {
        set({ isLoading: true, error: null })
        try {
            await updateDj(id, data) // Using the imported API function (needs import update)
            await get().fetchDjs()
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
            throw error
        }
    },

    removeDj: async (id) => {
        set({ isLoading: true, error: null })
        try {
            await deleteDj(id)
            await get().fetchDjs()
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
        }
    },

    fetchAiProviders: async () => {
        set({ isLoading: true, error: null })
        try {
            const aiProviders = await getAiProviders()
            set({ aiProviders, isLoading: false })
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
        }
    },

    addAiProvider: async (data) => {
        set({ isLoading: true, error: null })
        try {
            await createAiProvider(data)
            await get().fetchAiProviders()
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
            throw error
        }
    },

    updateAiProvider: async (id, data) => {
        set({ isLoading: true, error: null })
        try {
            await updateAiProvider(id, data)
            await get().fetchAiProviders()
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
            throw error
        }
    },

    removeAiProvider: async (id) => {
        set({ isLoading: true, error: null })
        try {
            await deleteAiProvider(id)
            await get().fetchAiProviders()
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false })
        }
    }
}))
