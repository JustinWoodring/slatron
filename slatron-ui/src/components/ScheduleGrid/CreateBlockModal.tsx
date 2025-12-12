import { useState, useEffect } from 'react'
import { useContentStore } from '../../stores/contentStore'

interface CreateBlockModalProps {
    isOpen: boolean
    onClose: () => void
    onSubmit: (data: any) => void
    initialData?: {
        day_of_week?: number
        start_time?: string
    }
}

export function CreateBlockModal({ isOpen, onClose, onSubmit, initialData }: CreateBlockModalProps) {
    const { content, fetchContent } = useContentStore()
    const [formData, setFormData] = useState({
        day_of_week: 0,
        start_time: '09:00:00',
        duration_minutes: 15,
        content_id: ''
    })

    useEffect(() => {
        if (isOpen) {
            fetchContent()
            if (initialData) {
                setFormData(prev => ({
                    ...prev,
                    day_of_week: initialData.day_of_week ?? prev.day_of_week,
                    start_time: initialData.start_time ?? prev.start_time
                }))
            }
        }
    }, [isOpen, initialData])

    if (!isOpen) return null

    const days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
            <div className="glass-panel p-6 rounded-xl w-full max-w-md border border-[var(--border-color)]">
                <h2 className="text-xl font-bold text-white mb-4">Add Schedule Block</h2>

                <form onSubmit={(e) => {
                    e.preventDefault()
                    // Ensure time has seconds
                    const startTime = formData.start_time.length === 5 ? formData.start_time + ':00' : formData.start_time

                    onSubmit({
                        ...formData,
                        start_time: startTime,
                        content_id: formData.content_id ? parseInt(formData.content_id) : null
                    })
                }} className="space-y-4">

                    {/* Day Selection */}
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Day</label>
                        <select
                            className="w-full bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
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
                                className="w-full bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                                value={formData.start_time}
                                onChange={(e) => setFormData({ ...formData, start_time: e.target.value })}
                            />
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Duration (min)</label>
                            <input
                                type="number"
                                className="w-full bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                                value={formData.duration_minutes}
                                onChange={(e) => setFormData({ ...formData, duration_minutes: parseInt(e.target.value) })}
                            />
                        </div>
                    </div>

                    {/* Content Selection */}
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Content (Optional)</label>
                        <select
                            className="w-full bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                            value={formData.content_id}
                            onChange={(e) => setFormData({ ...formData, content_id: e.target.value })}
                        >
                            <option value="">No Content (Placeholder)</option>
                            {content.map(item => (
                                <option key={item.id} value={item.id}>{item.title} ({item.content_type})</option>
                            ))}
                        </select>
                    </div>

                    <div className="flex gap-3 pt-4">
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex-1 px-4 py-2 rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:bg-[var(--bg-secondary)] transition-colors"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            className="flex-1 btn-primary"
                        >
                            Create Block
                        </button>
                    </div>
                </form>
            </div>
        </div>
    )
}
