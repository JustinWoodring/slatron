import type { ScheduleBlock } from '../hooks/useSchedules'

// True when (startTime + duration) intersects any block in the same context:
// a weekly block (day != null) collides only with blocks on the same weekday;
// a one-off block (date != null) only with blocks on the same date.
// Mirrors the server-side overlap check in schedules_api.rs.
export const blocksOverlap = (
    blocks: ScheduleBlock[],
    day: number | null,
    date: string | null,
    startTime: string,
    duration: number,
    excludeBlockId?: number
): boolean => {
    const [newHours, newMins] = startTime.split(':').map(Number)
    const newStartTotal = newHours * 60 + newMins
    const newEndTotal = newStartTotal + duration

    return blocks.some(b => {
        if (b.id === excludeBlockId) return false

        if (day !== null) {
            if (b.day_of_week !== day) return false
        } else if (date !== null) {
            if (b.specific_date !== date) return false
        }

        const [bHours, bMins] = b.start_time.split(':').map(Number)
        const bStartTotal = bHours * 60 + bMins
        const bEndTotal = bStartTotal + b.duration_minutes

        // (StartA < EndB) && (EndA > StartB)
        return newStartTotal < bEndTotal && newEndTotal > bStartTotal
    })
}
