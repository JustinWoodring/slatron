import React, { useState, useEffect } from 'react'
import { useScheduleStore } from '../../stores/scheduleStore'
import { useContentStore } from '../../stores/contentStore'
import { ContentPickerModal } from './ContentPickerModal'

interface EventEditorModalProps {
    isOpen: boolean
    onClose: () => void
    blockId?: number | null // If null, creating. If set, updating.
    scheduleId: number
}

export const EventEditorModal = ({ isOpen, onClose, blockId, scheduleId }: EventEditorModalProps) => {
    const { blocks, createBlock, updateBlock, checkOverlap } = useScheduleStore()
    const { content, fetchContent } = useContentStore()

    // Find block if editing
    const editingBlock = blockId ? blocks.find(b => b.id === blockId) : null

    const [formData, setFormData] = useState({
        specific_date: '',
        start_time: '12:00',
        duration_minutes: 60,
        content_id: '' as string | number
    })

    const [isContentPickerOpen, setIsContentPickerOpen] = useState(false)
    const [error, setError] = useState<string | null>(null)

    useEffect(() => {
        if (isOpen) {
            fetchContent()
        }
    }, [isOpen])

    useEffect(() => {
        if (isOpen) {
            if (editingBlock) {
                setFormData({
                    specific_date: editingBlock.specific_date || '',
                    start_time: editingBlock.start_time.slice(0, 5),
                    duration_minutes: editingBlock.duration_minutes,
                    content_id: editingBlock.content_id ?? ''
                })
            } else {
                // Defaults
                setFormData({
                    specific_date: new Date().toISOString().split('T')[0],
                    start_time: '12:00',
                    duration_minutes: 60,
                    content_id: '' as string | number
                })
            }
            setError(null)
        }
    }, [isOpen, blockId])

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        setError(null)

        const startTimeFull = formData.start_time.length === 5 ? formData.start_time + ':00' : formData.start_time

        // Overlap Check (One-off)
        const hasOverlap = checkOverlap(
            null, // day
            formData.specific_date,
            startTimeFull,
            formData.duration_minutes,
            editingBlock?.id
        )

        if (hasOverlap) {
            setError('This specific time overlaps with another event.')
            return;
        }

        try {
            const payload = {
                specific_date: formData.specific_date,
                start_time: startTimeFull,
                duration_minutes: formData.duration_minutes,
                content_id: formData.content_id ? Number(formData.content_id) : null,
                schedule_id: scheduleId
                // day_of_week is implicitly null for one-off? Or should be explicit?
                // Backend might require null.
            }

            if (editingBlock) {
                await updateBlock(scheduleId, editingBlock.id, payload)
            } else {
                await createBlock(scheduleId, payload)
            }
            onClose()
        } catch (e: any) {
            console.error("Failed to save event", e)
            setError(e.response?.status === 409 ? 'Backend rejected due to overlap.' : 'Failed to save event')
        }
    }

    if (!isOpen) return null

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border-color)] w-full max-w-md overflow-hidden animate-fade-in shadow-2xl">
                <div className="p-4 border-b border-[var(--border-color)] flex justify-between items-center bg-[var(--bg-tertiary)]">
                    <h2 className="text-lg font-bold text-white">
                        {editingBlock
                            ? (editingBlock.content_id
                                ? `Edit: ${content.find(c => c.id === editingBlock.content_id)?.title || 'Event'}`
                                : 'Edit Event')
                            : 'Add Event'}
                    </h2>
                    <button onClick={onClose} className="text-[var(--text-secondary)] hover:text-white transition-colors">
                        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-6 space-y-4">
                    {error && (
                        <div className="p-2 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
                            {error}
                        </div>
                    )}

                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Date</label>
                        <input
                            type="date"
                            required
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:border-indigo-500 outline-none"
                            value={formData.specific_date}
                            onChange={e => setFormData({ ...formData, specific_date: e.target.value })}
                        />
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Time</label>
                            <input
                                type="time"
                                required
                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:border-indigo-500 outline-none"
                                value={formData.start_time}
                                onChange={e => setFormData({ ...formData, start_time: e.target.value })}
                            />
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Duration (min)</label>
                            <input
                                type="number"
                                required
                                min="1"
                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:border-indigo-500 outline-none"
                                value={formData.duration_minutes}
                                onChange={e => setFormData({ ...formData, duration_minutes: parseInt(e.target.value) })}
                            />
                        </div>
                    </div>

                    {/* Content Picker */}
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Content</label>
                        <button
                            type="button"
                            onClick={() => setIsContentPickerOpen(true)}
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-left text-sm focus:outline-none hover:bg-[var(--bg-tertiary)] transition-colors flex items-center justify-between"
                        >
                            <div className="truncate text-white">
                                {formData.content_id ? (
                                    content.find(c => c.id == Number(formData.content_id))?.title || `Content #${formData.content_id}`
                                ) : (
                                    <span className="text-[var(--text-secondary)] italic">Select content...</span>
                                )}
                            </div>
                        </button>
                    </div>

                    <div className="flex justify-end gap-3 pt-4">
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-4 py-2 rounded-lg text-sm font-medium text-[var(--text-secondary)] hover:text-white hover:bg-[var(--bg-primary)] transition-colors"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            className="btn-primary"
                        >
                            Save Event
                        </button>
                    </div>
                </form>

                <ContentPickerModal
                    isOpen={isContentPickerOpen}
                    onClose={() => setIsContentPickerOpen(false)}
                    onSelect={(contentId) => {
                        setFormData({ ...formData, content_id: contentId || '' })
                        setIsContentPickerOpen(false)
                    }}
                    content={content}
                    currentId={formData.content_id}
                />
            </div>
        </div>
    )
}
