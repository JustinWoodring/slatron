import React, { useState, useEffect } from 'react'
import { useContentStore } from '../../stores/contentStore'
import { useScheduleStore } from '../../stores/scheduleStore'

interface BlockEditorSidebarProps {
    blockId: number | null
    onClose: () => void
}

export function BlockEditorSidebar({ blockId, onClose }: BlockEditorSidebarProps) {
    const { blocks, updateBlock, deleteBlock, selectedScheduleId } = useScheduleStore()
    const { content, fetchContent } = useContentStore()

    // Find the block from the store
    const block = blocks.find(b => b.id === blockId)

    const [formData, setFormData] = useState({
        day_of_week: 0,
        start_time: '',
        duration_minutes: 15,
        content_id: '' as string | number
    })

    useEffect(() => {
        fetchContent()
    }, [])

    useEffect(() => {
        if (block) {
            setFormData({
                day_of_week: block.day_of_week ?? 0,
                start_time: block.start_time,
                duration_minutes: block.duration_minutes,
                content_id: block.content_id ?? ''
            })
        }
    }, [block])

    const handleUpdate = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!block || !selectedScheduleId) return

        try {
            // Ensure time has seconds
            const startTime = formData.start_time.length === 5 ? formData.start_time + ':00' : formData.start_time

            await updateBlock(selectedScheduleId, block.id, {
                ...formData,
                start_time: startTime,
                content_id: formData.content_id ? Number(formData.content_id) : null
            })
            // Optional: visual feedback
        } catch (error) {
            console.error("Failed to update block", error)
        }
    }

    const handleDelete = async () => {
        if (!block || !selectedScheduleId) return
        if (window.confirm("Are you sure you want to delete this block?")) {
            await deleteBlock(selectedScheduleId, block.id)
            onClose()
        }
    }

    if (!block) {
        return (
            <div className={`fixed right-0 top-0 bottom-0 w-80 bg-[var(--bg-secondary)] border-l border-[var(--border-color)] transform transition-transform duration-300 z-40 p-4 ${blockId ? 'translate-x-0' : 'translate-x-full'}`}>
                {/* Empty state or transition handler */}
            </div>
        )
    }

    const days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']

    return (
        <div className="fixed right-0 top-0 bottom-0 w-96 bg-[var(--bg-secondary)] border-l border-[var(--border-color)] shadow-xl z-40 flex flex-col pt-16 animate-slide-in-right">
            <div className="flex items-center justify-between p-4 border-b border-[var(--border-color)]">
                <h2 className="text-lg font-bold text-white">Edit Event</h2>
                <button onClick={onClose} className="text-[var(--text-secondary)] hover:text-white">
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <form onSubmit={handleUpdate} className="flex-1 overflow-y-auto p-4 space-y-6">
                {/* Day Selection */}
                <div>
                    <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Day</label>
                    <select
                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                        value={formData.day_of_week}
                        onChange={(e) => setFormData({ ...formData, day_of_week: parseInt(e.target.value) })}
                    >
                        {days.map((day, index) => (
                            <option key={index} value={index}>{day}</option>
                        ))}
                    </select>
                </div>

                {/* Time Selection */}
                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Start Time</label>
                        <input
                            type="time"
                            step="1"
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                            value={formData.start_time}
                            onChange={(e) => setFormData({ ...formData, start_time: e.target.value })}
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Duration (min)</label>
                        <input
                            type="number"
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                            value={formData.duration_minutes}
                            onChange={(e) => setFormData({ ...formData, duration_minutes: parseInt(e.target.value) })}
                        />
                    </div>
                </div>

                {/* Content Selection */}
                <div>
                    <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Content (Optional)</label>
                    <select
                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                        value={formData.content_id}
                        onChange={(e) => setFormData({ ...formData, content_id: e.target.value })}
                    >
                        <option value="">No Content (Placeholder)</option>
                        {content.map(item => (
                            <option key={item.id} value={item.id}>{item.title} ({item.content_type})</option>
                        ))}
                    </select>
                </div>
            </form>

            <div className="p-4 border-t border-[var(--border-color)] bg-[var(--bg-secondary)] flex gap-3">
                <button
                    type="button"
                    onClick={handleDelete}
                    className="px-4 py-2 rounded-lg border border-red-500/30 text-red-400 hover:bg-red-500/10 transition-colors"
                >
                    Delete
                </button>
                <button
                    onClick={handleUpdate}
                    className="flex-1 btn-primary"
                >
                    Save Changes
                </button>
            </div>
        </div>
    )
}
