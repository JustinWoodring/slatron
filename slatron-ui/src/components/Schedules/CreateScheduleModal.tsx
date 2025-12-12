import React, { useState } from 'react'
import { useScheduleStore } from '../../stores/scheduleStore'

interface CreateScheduleModalProps {
    isOpen: boolean
    onClose: () => void
}

export const CreateScheduleModal = ({ isOpen, onClose }: CreateScheduleModalProps) => {
    const { createSchedule } = useScheduleStore()
    const [formData, setFormData] = useState({
        name: '',
        description: '',
        schedule_type: 'weekly', // Default
        priority: 0,
        is_active: true
    })

    if (!isOpen) return null

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        try {
            await createSchedule(formData)
            onClose()
            // Reset form
            setFormData({
                name: '',
                description: '',
                schedule_type: 'weekly',
                priority: 0,
                is_active: true
            })
        } catch (error) {
            console.error('Failed to create schedule:', error)
            alert('Failed to create schedule')
        }
    }

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border-color)] w-full max-w-md overflow-hidden animate-fade-in shadow-2xl">
                <div className="p-6 border-b border-[var(--border-color)] flex justify-between items-center bg-[var(--bg-tertiary)]">
                    <h2 className="text-xl font-bold text-white">Create New Schedule</h2>
                    <button onClick={onClose} className="text-[var(--text-secondary)] hover:text-white transition-colors">
                        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-6 space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Schedule Name</label>
                        <input
                            type="text"
                            required
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white focus:border-indigo-500 outline-none"
                            value={formData.name}
                            onChange={e => setFormData({ ...formData, name: e.target.value })}
                            placeholder="e.g. Prime Time"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Schedule Type</label>
                        <div className="grid grid-cols-2 gap-3">
                            <button
                                type="button"
                                onClick={() => setFormData({ ...formData, schedule_type: 'weekly' })}
                                className={`p-3 rounded-lg border text-sm font-medium transition-all ${formData.schedule_type === 'weekly'
                                        ? 'bg-indigo-500/20 border-indigo-500 text-white'
                                        : 'bg-[var(--bg-primary)] border-[var(--border-color)] text-[var(--text-secondary)] hover:border-indigo-500/50'
                                    }`}
                            >
                                Weekly
                                <span className="block text-xs font-normal opacity-70 mt-1">Recurring 7-day plan</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => setFormData({ ...formData, schedule_type: 'one_off' })}
                                className={`p-3 rounded-lg border text-sm font-medium transition-all ${formData.schedule_type === 'one_off'
                                        ? 'bg-indigo-500/20 border-indigo-500 text-white'
                                        : 'bg-[var(--bg-primary)] border-[var(--border-color)] text-[var(--text-secondary)] hover:border-indigo-500/50'
                                    }`}
                            >
                                One-off
                                <span className="block text-xs font-normal opacity-70 mt-1">Specific date events</span>
                            </button>
                        </div>
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Description</label>
                        <textarea
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white focus:border-indigo-500 outline-none resize-none h-24"
                            value={formData.description}
                            onChange={e => setFormData({ ...formData, description: e.target.value })}
                            placeholder="Optional description..."
                        />
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
                            Create Schedule
                        </button>
                    </div>
                </form>
            </div>
        </div>
    )
}
