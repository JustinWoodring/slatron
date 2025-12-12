import React, { useState, useEffect } from 'react'
import { useScheduleStore } from '../../stores/scheduleStore'
import { useContentStore } from '../../stores/contentStore'
// import { BlockEditorPopover } from './BlockEditorPopover' // Reusing edit logic if possible, or build new?
// Actually popover needs X/Y. List view might be better with a Modal or just reuse Popover with fake pos?
// Better to simple inline edit or modal. Let's use a simpler Modal for One-Offs.
// Or just reuse BlockEditorPopover logic but wrap it?

// Let's create a dedicated simplified "EventModal" later if needed. 
// For now, I'll iterate on the list.
import { EventEditorModal } from './EventEditorModal'

export const OneOffScheduleList = () => {
    const { blocks, deleteBlock, selectedScheduleId } = useScheduleStore()
    const { content, fetchContent } = useContentStore()

    useEffect(() => {
        fetchContent()
    }, [])

    const [isModalOpen, setIsModalOpen] = useState(false)
    const [editingBlockId, setEditingBlockId] = useState<number | null>(null)

    // Sort blocks by date + time
    const sortedBlocks = [...blocks].sort((a, b) => {
        const dateA = a.specific_date || '9999-99-99'
        const dateB = b.specific_date || '9999-99-99'
        if (dateA !== dateB) return dateA.localeCompare(dateB)
        return a.start_time.localeCompare(b.start_time)
    })

    const handleDelete = async (e: React.MouseEvent, blockId: number) => {
        e.stopPropagation()
        if (!selectedScheduleId) return;
        if (confirm('Delete this event?')) {
            await deleteBlock(selectedScheduleId, blockId)
        }
    }

    const handleEdit = (blockId: number) => {
        setEditingBlockId(blockId)
        setIsModalOpen(true)
    }

    const handleCreate = () => {
        setEditingBlockId(null)
        setIsModalOpen(true)
    }

    if (!selectedScheduleId) return null;

    return (
        <div className="p-4 overflow-y-auto h-full relative">
            <div className="flex justify-end mb-4">
                <button
                    onClick={handleCreate}
                    className="btn-primary flex items-center gap-2"
                >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                    </svg>
                    Add Event
                </button>
            </div>

            <div className="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
                {sortedBlocks.map(block => (
                    <div
                        key={block.id}
                        onClick={() => handleEdit(block.id)}
                        className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl p-4 flex flex-col gap-2 relative group hover:border-indigo-500/50 transition-colors cursor-pointer"
                    >
                        <div className="flex justify-between items-start">
                            <div>
                                <div className="text-sm font-bold text-indigo-400">
                                    {block.specific_date ? new Date(block.specific_date).toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' }) : 'No Date'}
                                </div>
                                <div className="text-2xl font-light text-white">
                                    {block.start_time.slice(0, 5)}
                                </div>
                                <div className="text-xs text-[var(--text-secondary)]">
                                    {block.duration_minutes} min
                                </div>
                            </div>
                            <button
                                onClick={(e) => handleDelete(e, block.id)}
                                className="opacity-0 group-hover:opacity-100 p-1 text-red-400 hover:bg-red-500/10 rounded transition-all"
                            >
                                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                            </button>
                        </div>

                        <div className="mt-2 pt-2 border-t border-[var(--border-color)]">
                            <div className="text-sm text-white truncate">
                                {block.content_id
                                    ? content.find(c => c.id === block.content_id)?.title || 'Unknown Content'
                                    : <span className="text-[var(--text-secondary)] italic">No Content</span>
                                }
                            </div>
                        </div>
                    </div>
                ))}
            </div>

            {sortedBlocks.length === 0 && (
                <div className="text-center text-[var(--text-secondary)] mt-20">
                    No events scheduled.
                </div>
            )}

            <EventEditorModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                blockId={editingBlockId}
                scheduleId={selectedScheduleId}
            />
        </div>
    )
}
