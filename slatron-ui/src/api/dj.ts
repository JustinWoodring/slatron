import { apiClient } from './client'

export interface DjProfile {
    id: number
    name: string
    personality_prompt: string
    voice_config_json: string
    context_depth: number
    voice_provider_id?: number | null
    llm_provider_id?: number | null
    context_script_ids?: string | null
    created_at: string
    talkativeness?: number
}

export interface NewDjProfile {
    name: string
    voice_config_json: string
    personality_prompt: string
    context_depth: number
    voice_provider_id?: number | null
    llm_provider_id?: number | null
    context_script_ids?: string | null;
    talkativeness: number;
}

export interface DjMemory {
    id: number;
    dj_id: number;
    memory_type: string;
    content: string;
    importance_score: number;
    happened_at: string;
    created_at: string;
}

export interface NewDjMemory {
    dj_id?: number | null; // Optional if derived from context/path
    memory_type: string;
    content: string;
    importance_score: number;
    happened_at: string;
}

export interface UpdateDjMemory {
    content?: string;
    importance_score?: number;
    memory_type?: string;
}

export interface AiProvider {
    id: number
    name: string
    provider_type: string
    endpoint_url: string | null
    model_name: string | null
    is_active: boolean
    provider_category: string
}

export interface NewAiProvider {
    name: string
    provider_type: string
    api_key?: string
    endpoint_url?: string
    model_name?: string
    is_active: boolean
    provider_category: string
}

export const getDjs = () => apiClient.get<DjProfile[]>('/api/djs').then(res => res.data)
export const createDj = (data: NewDjProfile) => apiClient.post<DjProfile>('/api/djs', data).then(res => res.data)
export const deleteDj = (id: number) => apiClient.delete(`/api/djs/${id}`)
export const getDj = (id: number) => apiClient.get<DjProfile>(`/api/djs/${id}`).then(res => res.data)
export const updateDj = (id: number, data: Partial<NewDjProfile>) => apiClient.put<DjProfile>(`/api/djs/${id}`, data).then(res => res.data)

export const getAiProviders = async () => {
    const res = await apiClient.get<AiProvider[]>('/api/ai-providers');
    return res.data;
};

export const createAiProvider = async (provider: NewAiProvider) => {
    const res = await apiClient.post('/api/ai-providers', provider);
    return res.data;
};

export const updateAiProvider = async (id: number, provider: Partial<AiProvider>) => {
    const res = await apiClient.put(`/api/ai-providers/${id}`, provider);
    return res.data;
};

export const deleteAiProvider = async (id: number) => {
    await apiClient.delete(`/api/ai-providers/${id}`);
};

// Memories
export const getDjMemories = async (djId: number): Promise<DjMemory[]> => {
    const res = await apiClient.get(`/api/djs/${djId}/memories`);
    return res.data;
};

export const createDjMemory = async (djId: number, memory: NewDjMemory): Promise<DjMemory> => {
    const res = await apiClient.post(`/api/djs/${djId}/memories`, memory);
    return res.data;
};

export const updateDjMemory = async (memoryId: number, memory: UpdateDjMemory): Promise<DjMemory> => {
    const res = await apiClient.put(`/api/memories/${memoryId}`, memory);
    return res.data;
};

export const deleteDjMemory = async (memoryId: number) => {
    await apiClient.delete(`/api/memories/${memoryId}`);
};
