import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../api/client'

export interface Schedule {
    id: number
    name: string
    description: string | null
    schedule_type: string
    priority: number
    is_active: boolean
    created_at: string
    updated_at: string
}

export interface ScheduleBlock {
    id: number
    schedule_id: number
    content_id: number | null
    day_of_week: number | null
    specific_date: string | null
    start_time: string
    duration_minutes: number
    script_id: number | null
    dj_id: number | null
}

export interface NodeAssignedSchedule {
    id: number
    name: string
    priority: number
}

export const SCHEDULES_QUERY_KEY = ['schedules']
export const blocksQueryKey = (scheduleId: number) => ['schedules', scheduleId, 'blocks']

const fetchSchedules = async (): Promise<Schedule[]> => {
    const response = await apiClient.get('/api/schedules')
    return response.data
}

export const useSchedules = () =>
    useQuery({ queryKey: SCHEDULES_QUERY_KEY, queryFn: fetchSchedules })

export const useCreateSchedule = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (data: Partial<Schedule>) => {
            const response = await apiClient.post('/api/schedules', data)
            return response.data as Schedule
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: SCHEDULES_QUERY_KEY }),
    })
}

export const useUpdateSchedule = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ id, data }: { id: number; data: Partial<Schedule> }) => {
            const response = await apiClient.put(`/api/schedules/${id}`, data)
            return response.data as Schedule
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: SCHEDULES_QUERY_KEY }),
    })
}

export const useDeleteSchedule = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            await apiClient.delete(`/api/schedules/${id}`)
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: SCHEDULES_QUERY_KEY }),
    })
}

const fetchBlocks = async (scheduleId: number): Promise<ScheduleBlock[]> => {
    const response = await apiClient.get(`/api/schedules/${scheduleId}/blocks`)
    return response.data
}

export const useBlocks = (scheduleId: number | null) =>
    useQuery({
        queryKey: blocksQueryKey(scheduleId ?? 0),
        queryFn: () => fetchBlocks(scheduleId as number),
        enabled: scheduleId !== null,
    })

const blockPayload = (scheduleId: number, data: Partial<ScheduleBlock>) => ({
    ...data,
    schedule_id: scheduleId,
})

export const useCreateBlock = (scheduleId: number) => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (data: Partial<ScheduleBlock>) => {
            const response = await apiClient.post(
                `/api/schedules/${scheduleId}/blocks`,
                blockPayload(scheduleId, data)
            )
            return response.data as ScheduleBlock
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: blocksQueryKey(scheduleId) }),
    })
}

export const useUpdateBlock = (scheduleId: number) => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ blockId, data }: { blockId: number; data: Partial<ScheduleBlock> }) => {
            const response = await apiClient.put(
                `/api/schedules/${scheduleId}/blocks/${blockId}`,
                blockPayload(scheduleId, data)
            )
            return response.data as ScheduleBlock
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: blocksQueryKey(scheduleId) }),
    })
}

export const useDeleteBlock = (scheduleId: number) => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (blockId: number) => {
            await apiClient.delete(`/api/schedules/${scheduleId}/blocks/${blockId}`)
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: blocksQueryKey(scheduleId) }),
    })
}

const fetchNodeAssignedSchedules = async (nodeId: number): Promise<NodeAssignedSchedule[]> => {
    const response = await apiClient.get(`/api/nodes/${nodeId}/schedule`)
    return response.data.assigned_schedules || []
}

export const useNodeAssignedSchedules = (nodeId: number | null) =>
    useQuery({
        queryKey: ['nodes', nodeId, 'assigned-schedules'],
        queryFn: () => fetchNodeAssignedSchedules(nodeId as number),
        enabled: nodeId !== null,
    })

export const useUpdateNodeSchedules = () => {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ nodeId, scheduleIds }: { nodeId: number; scheduleIds: number[] }) => {
            await apiClient.put(`/api/nodes/${nodeId}/schedules`, { schedule_ids: scheduleIds })
        },
        onSuccess: (_data, variables) =>
            queryClient.invalidateQueries({ queryKey: ['nodes', variables.nodeId, 'assigned-schedules'] }),
    })
}
