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
  dj_id: number | null
}

interface ScheduleStore {
  schedules: Schedule[]
  blocks: ScheduleBlock[]
  selectedScheduleId: number | null

  fetchSchedules: () => Promise<void>
  fetchBlocks: (scheduleId: number) => Promise<void>

  createSchedule: (data: Partial<Schedule>) => Promise<void>
  updateSchedule: (id: number, data: Partial<Schedule>) => Promise<void>
  deleteSchedule: (id: number) => Promise<void>

  createBlock: (scheduleId: number, blockData: Partial<ScheduleBlock>) => Promise<ScheduleBlock>
  updateBlock: (scheduleId: number, blockId: number, data: Partial<ScheduleBlock>) => Promise<void>
  deleteBlock: (scheduleId: number, blockId: number) => Promise<void>

  nodeAssignedSchedules: Schedule[]
  fetchNodeAssignedSchedules: (nodeId: number) => Promise<void>

  updateNodeSchedules: (nodeId: number, scheduleIds: number[]) => Promise<void>
  setSelectedSchedule: (id: number | null) => void
  checkOverlap: (day: number | null, date: string | null, startTime: string, duration: number, excludeBlockId?: number) => boolean
}

export const useScheduleStore = create<ScheduleStore>((set, get) => ({
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
    const payload = { ...data, schedule_id: scheduleId };
    const response = await apiClient.post<ScheduleBlock>(`/api/schedules/${scheduleId}/blocks`, payload)
    await get().fetchBlocks(scheduleId)
    return response.data
  },

  updateBlock: async (scheduleId, blockId, data) => {
    // Ensure schedule_id is included, just like createBlock
    const payload = { ...data, schedule_id: scheduleId };

    // We expect the backend to return the updated block
    const response = await apiClient.put(
      `/api/schedules/${scheduleId}/blocks/${blockId}`,
      payload
    )

    // Update local state immediately (optimistic-ish)
    set({
      blocks: get().blocks.map((b) =>
        b.id === blockId ? response.data : b
      ),
    })

    // Optionally refetch to ensure consistency (like createBlock does)
    // This helps if there are side effects or sorting/collision logic on backend
    // await get().fetchBlocks(scheduleId) 
  },

  deleteBlock: async (scheduleId, blockId) => {
    await apiClient.delete(`/api/schedules/${scheduleId}/blocks/${blockId}`)
    set({
      blocks: get().blocks.filter((b) => b.id !== blockId),
    })
  },

  nodeAssignedSchedules: [],
  fetchNodeAssignedSchedules: async (nodeId) => {
    const response = await apiClient.get<any>(`/api/nodes/${nodeId}/schedule`)
    set({ nodeAssignedSchedules: response.data.assigned_schedules || [] })
  },



  updateNodeSchedules: async (nodeId, scheduleIds) => {
    await apiClient.put(`/api/nodes/${nodeId}/schedules`, { schedule_ids: scheduleIds })
  },

  setSelectedSchedule: (id) => set({ selectedScheduleId: id }),

  checkOverlap: (day: number | null, date: string | null, startTime: string, duration: number, excludeBlockId?: number) => {
    const blocks = get().blocks;
    const [newHours, newMins] = startTime.split(':').map(Number);
    const newStartTotal = newHours * 60 + newMins;
    const newEndTotal = newStartTotal + duration;

    return blocks.some(b => {
      if (b.id === excludeBlockId) return false;

      // Check context match
      // If we are checking a Weekly block (day != null), only compare with blocks that have matching day
      if (day !== null) {
        if (b.day_of_week !== day) return false;
      }
      // If we are checking a One-Off block (date != null), only compare with blocks that have matching date
      else if (date !== null) {
        if (b.specific_date !== date) return false;
      }

      const [bHours, bMins] = b.start_time.split(':').map(Number);
      const bStartTotal = bHours * 60 + bMins;
      const bEndTotal = bStartTotal + b.duration_minutes;

      // (StartA < EndB) && (EndA > StartB)
      return newStartTotal < bEndTotal && newEndTotal > bStartTotal;
    });
  }
}))
