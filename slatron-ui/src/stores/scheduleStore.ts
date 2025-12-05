import { create } from 'zustand'
import { apiClient } from '../api/client'

interface Schedule {
  id: number
  name: string
  description: string | null
  schedule_type: string
  priority: number
  is_active: boolean
  created_at: string
  updated_at: string
}

interface ScheduleBlock {
  id: number
  schedule_id: number
  content_id: number | null
  day_of_week: number | null
  specific_date: string | null
  start_time: string
  duration_minutes: number
  script_id: number | null
  created_at: string
  updated_at: string
}

interface ScheduleState {
  schedules: Schedule[]
  blocks: ScheduleBlock[]
  selectedScheduleId: number | null

  fetchSchedules: () => Promise<void>
  fetchBlocks: (scheduleId: number) => Promise<void>
  createSchedule: (data: Partial<Schedule>) => Promise<void>
  updateSchedule: (id: number, data: Partial<Schedule>) => Promise<void>
  deleteSchedule: (id: number) => Promise<void>

  createBlock: (scheduleId: number, data: Partial<ScheduleBlock>) => Promise<void>
  updateBlock: (scheduleId: number, blockId: number, data: Partial<ScheduleBlock>) => Promise<void>
  deleteBlock: (scheduleId: number, blockId: number) => Promise<void>

  setSelectedSchedule: (id: number | null) => void
}

export const useScheduleStore = create<ScheduleState>((set, get) => ({
  schedules: [],
  blocks: [],
  selectedScheduleId: null,

  fetchSchedules: async () => {
    const response = await apiClient.get('/api/schedules')
    set({ schedules: response.data })
  },

  fetchBlocks: async (scheduleId: number) => {
    const response = await apiClient.get(`/api/schedules/${scheduleId}/blocks`)
    set({ blocks: response.data, selectedScheduleId: scheduleId })
  },

  createSchedule: async (data) => {
    const response = await apiClient.post('/api/schedules', data)
    set({ schedules: [...get().schedules, response.data] })
  },

  updateSchedule: async (id, data) => {
    const response = await apiClient.put(`/api/schedules/${id}`, data)
    set({
      schedules: get().schedules.map((s) =>
        s.id === id ? response.data : s
      ),
    })
  },

  deleteSchedule: async (id) => {
    await apiClient.delete(`/api/schedules/${id}`)
    set({
      schedules: get().schedules.filter((s) => s.id !== id),
    })
  },

  createBlock: async (scheduleId, data) => {
    const response = await apiClient.post(`/api/schedules/${scheduleId}/blocks`, data)
    set({ blocks: [...get().blocks, response.data] })
  },

  updateBlock: async (scheduleId, blockId, data) => {
    const response = await apiClient.put(
      `/api/schedules/${scheduleId}/blocks/${blockId}`,
      data
    )
    set({
      blocks: get().blocks.map((b) =>
        b.id === blockId ? response.data : b
      ),
    })
  },

  deleteBlock: async (scheduleId, blockId) => {
    await apiClient.delete(`/api/schedules/${scheduleId}/blocks/${blockId}`)
    set({
      blocks: get().blocks.filter((b) => b.id !== blockId),
    })
  },

  setSelectedSchedule: (id) => set({ selectedScheduleId: id }),
}))
