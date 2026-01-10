import React, { useState, useEffect, useRef } from 'react'
import { useContentStore } from '../../stores/contentStore'
import { useScheduleStore } from '../../stores/scheduleStore'
import { useDjStore } from '../../stores/djStore'
import { ContentPickerModal } from './ContentPickerModal'

interface BlockEditorPopoverProps {
    blockId: number | null
    onClose: () => void
    position: { x: number, y: number } | null
    readOnly?: boolean
}

export function BlockEditorPopover({ blockId, onClose, position, readOnly = false }: BlockEditorPopoverProps) {
    const { blocks, updateBlock, deleteBlock, selectedScheduleId } = useScheduleStore()
    const { content, fetchContent } = useContentStore()
    const { djs, fetchDjs } = useDjStore()
    const popoverRef = useRef<HTMLDivElement>(null)

    // Find the block from the store
    const block = blocks.find(b => b.id === blockId)

    const [formData, setFormData] = useState({
        day_of_week: 0,
        start_time: '',
        duration_minutes: 15,
        content_id: '' as string | number,
        dj_id: '' as string | number
    })

    const [blockType, setBlockType] = useState<'content' | 'dj'>('content')

    useEffect(() => {
        fetchContent()
        fetchDjs()
    }, [])

    useEffect(() => {
        if (block) {
            const type = block.content_id ? 'content' : (block.dj_id ? 'dj' : 'content')
            setBlockType(type)

            setFormData({
                day_of_week: block.day_of_week ?? 0,
                start_time: block.start_time,
                duration_minutes: block.duration_minutes,
                content_id: block.content_id ?? '',
                dj_id: block.dj_id ?? ''
            })
        }
    }, [block])

    // Close on click outside
    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
                onClose()
            }
        }
        document.addEventListener("mousedown", handleClickOutside)
        return () => {
            document.removeEventListener("mousedown", handleClickOutside)
        }
    }, [onClose])

    // Adjust position to stay on screen
    const [style, setStyle] = useState<React.CSSProperties>({ opacity: 0 })

    useEffect(() => {
        if (!position || !popoverRef.current) return

        const updatePosition = () => {
            let top = position.y
            let left = position.x

            // Dimensions
            const popoverWidth = 384 // w-96
            const popoverHeight = popoverRef.current?.offsetHeight || 450 // Approximate or measured
            const padding = 20

            if (typeof window !== 'undefined') {
                const { innerWidth, innerHeight } = window

                // Horizontal Flip: If not enough space on right, flip to left of cursor
                if (left + popoverWidth + padding > innerWidth) {
                    // Try placing it to the left of the cursor
                    left = left - popoverWidth
                }

                // Vertical Flip: If not enough space below, flip up
                if (top + popoverHeight + padding > innerHeight) {
                    top = top - popoverHeight
                }

                // Hard Clamp: Ensure it never goes off-screen top/left
                left = Math.max(padding, Math.min(left, innerWidth - popoverWidth - padding))
                top = Math.max(padding, Math.min(top, innerHeight - popoverHeight - padding))
            }

            setStyle({ top, left, opacity: 1, transition: 'opacity 0.1s ease-in' })
        }

        updatePosition()
        // Recalculate if window resizes
        window.addEventListener('resize', updatePosition)
        return () => window.removeEventListener('resize', updatePosition)

    }, [position])

    const getPositionStyle = () => style


    const [error, setError] = useState<string | null>(null)
    const [isDeleting, setIsDeleting] = useState(false)
    const [isContentPickerOpen, setIsContentPickerOpen] = useState(false)

    const handleUpdate = async (e: React.FormEvent) => {
        e.preventDefault()
        if (readOnly) return
        setError(null)
        if (!block || !selectedScheduleId) return

        try {
            const startTime = formData.start_time.length === 5 ? formData.start_time + ':00' : formData.start_time

            const hasOverlap = useScheduleStore.getState().checkOverlap(
                formData.day_of_week,
                null, // Date (Weekly)
                startTime,
                formData.duration_minutes,
                block.id
            )

            if (hasOverlap) {
                setError("Overlaps with existing block")
                return
            }

            const payload = {
                ...formData,
                start_time: startTime,
                content_id: blockType === 'content' && formData.content_id ? Number(formData.content_id) : null,
                dj_id: blockType === 'dj' && formData.dj_id ? Number(formData.dj_id) : null
                // schedule_id injected by store
            }

            await updateBlock(selectedScheduleId, block.id, payload)
            // Close after explicit save? Google Calendar closes.
            onClose()
        } catch (error: any) {
            console.error("Failed to update block", error)
            setError(error.message || "Failed to update block")
        }
    }

    const handleDelete = async (e: React.MouseEvent) => {
        e.preventDefault()
        e.stopPropagation()
        if (readOnly) return
        setError(null)
        if (!block || !selectedScheduleId) return

        try {
            console.log(`Deleting block ${block.id} from schedule ${selectedScheduleId}`)
            await deleteBlock(selectedScheduleId, block.id)
            onClose()
        } catch (error: any) {
            console.error("Failed to delete block", error)
            setError(error.message || "Failed to delete block")
            setIsDeleting(false)
        }
    }

    if (!block || !position) return null

    const days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']

    return (
        <div
            ref={popoverRef}
            className="fixed z-50 w-96 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-2xl animate-fade-in"
            style={getPositionStyle()}
        >
            <div className="flex items-center justify-between p-4 border-b border-[var(--border-color)] bg-[var(--bg-tertiary)] rounded-t-xl" >
                <h2 className="text-sm font-bold text-white max-w-[200px] truncate">{readOnly ? 'Event Details' : 'Edit Event'}</h2>
                <button onClick={onClose} className="text-[var(--text-secondary)] hover:text-white">
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <form onSubmit={handleUpdate} className="p-4 space-y-4">
                {error && (
                    <div className="p-2 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-xs">
                        {error}
                    </div>
                )}

                {/* Type Selection Tabs */}
                {!readOnly && (
                    <div className="flex bg-[var(--bg-primary)] p-1 rounded-lg">
                        <button
                            type="button"
                            onClick={() => setBlockType('content')}
                            className={`flex-1 py-1.5 text-xs font-medium rounded-md transition-all ${blockType === 'content'
                                    ? 'bg-[var(--accent-primary)] text-white shadow-sm'
                                    : 'text-[var(--text-secondary)] hover:text-white'
                                }`}
                        >
                            Specific Content
                        </button>
                        <button
                            type="button"
                            onClick={() => setBlockType('dj')}
                            className={`flex-1 py-1.5 text-xs font-medium rounded-md transition-all ${blockType === 'dj'
                                    ? 'bg-[var(--accent-primary)] text-white shadow-sm'
                                    : 'text-[var(--text-secondary)] hover:text-white'
                                }`}
                        >
                            DJ Block (Auto)
                        </button>
                    </div>
                )}

                {/* Day Selection */}
                <div>
                    <label className="block text-xs font-medium text-[var(--text-secondary)] mb-1">Day</label>
                    <select
                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white text-sm focus:outline-none focus:border-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                        value={formData.day_of_week}
                        onChange={(e) => setFormData({ ...formData, day_of_week: parseInt(e.target.value) })}
                        disabled={readOnly}
                    >
                        {days.map((day, index) => (
                            <option key={index} value={index}>{day}</option>
                        ))}
                    </select>
                </div>

                {/* Time Selection */}
                <div className="grid grid-cols-2 gap-3">
                    <div>
                        <label className="block text-xs font-medium text-[var(--text-secondary)] mb-1">Start Time</label>
                        <input
                            type="time"
                            step="1"
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white text-sm focus:outline-none focus:border-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            value={formData.start_time}
                            onChange={(e) => setFormData({ ...formData, start_time: e.target.value })}
                            disabled={readOnly}
                        />
                    </div>
                    <div>
                        <label className="block text-xs font-medium text-[var(--text-secondary)] mb-1">Duration (min)</label>
                        <input
                            type="number"
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white text-sm focus:outline-none focus:border-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            value={formData.duration_minutes}
                            onChange={(e) => setFormData({ ...formData, duration_minutes: parseInt(e.target.value) })}
                            disabled={readOnly}
                        />
                    </div>
                </div>

                {/* Conditional Inputs */}
                {blockType === 'dj' && (
                    <div className="animate-fade-in">
                        <label className="block text-xs font-medium text-[var(--text-secondary)] mb-1">DJ Personality</label>
                        <select
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white text-sm focus:outline-none focus:border-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            value={formData.dj_id}
                            onChange={(e) => setFormData({ ...formData, dj_id: e.target.value })}
                            disabled={readOnly}
                        >
                            <option value="">Select a DJ...</option>
                            {djs.map((dj) => (
                                <option key={dj.id} value={dj.id}>{dj.name}</option>
                            ))}
                        </select>
                        <p className="mt-1 text-[10px] text-[var(--text-secondary)]">
                            The DJ will automatically select music and patter based on their profile.
                        </p>
                    </div>
                )}

                {blockType === 'content' && (
                    <div className="animate-fade-in">
                        <label className="block text-xs font-medium text-[var(--text-secondary)] mb-1">Fixed Content</label>
                        <button
                            type="button"
                            onClick={() => !readOnly && setIsContentPickerOpen(true)}
                            className={`w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-left text-sm focus:outline-none focus:border-indigo-500 transition-colors flex items-center justify-between group ${readOnly ? 'opacity-50 cursor-not-allowed' : 'hover:bg-[var(--bg-tertiary)]'}`}
                            disabled={readOnly}
                        >
                            <div className="truncate text-white">
                                {formData.content_id ? (
                                    content.find(c => c.id == Number(formData.content_id))?.title || `Content #${formData.content_id}`
                                ) : (
                                    <span className="text-[var(--text-secondary)] italic">Select content...</span>
                                )}
                            </div>
                            {!readOnly && (
                                <svg className="w-4 h-4 text-[var(--text-secondary)] group-hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                                </svg>
                            )}
                        </button>
                    </div>
                )}

                {!readOnly && (
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
                )}

                <div className="pt-2 flex gap-2">
                    {readOnly ? (
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex-1 btn-secondary text-sm py-1.5"
                        >
                            Close
                        </button>
                    ) : (
                        <>
                            {isDeleting ? (
                                <>
                                    <button
                                        type="button"
                                        onClick={() => setIsDeleting(false)}
                                        className="px-3 py-1.5 rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:text-white transition-colors text-sm"
                                    >
                                        Cancel
                                    </button>
                                    <button
                                        type="button"
                                        onClick={handleDelete}
                                        className="px-3 py-1.5 rounded-lg bg-red-500 text-white hover:bg-red-600 transition-colors text-sm"
                                    >
                                        Confirm Delete
                                    </button>
                                </>
                            ) : (
                                <button
                                    type="button"
                                    onClick={() => setIsDeleting(true)}
                                    className="px-3 py-1.5 rounded-lg border border-red-500/30 text-red-400 hover:bg-red-500/10 transition-colors text-sm"
                                >
                                    Delete
                                </button>
                            )}

                            {!isDeleting && (
                                <button
                                    type="submit"
                                    className="flex-1 btn-primary text-sm py-1.5"
                                >
                                    Save
                                </button>
                            )}
                        </>
                    )}
                </div>
            </form>
        </div>
    )
}
