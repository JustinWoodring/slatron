import { useMemo, useEffect, useRef } from 'react'

interface ScheduleBlock {
    id: number
    start_time: string // UTC "HH:MM:SS"
    duration_minutes: number
    content_id: number
    specific_date?: string
    source_schedule_name?: string
}

interface Content {
    id: number
    title: string
    duration_minutes: number
}

interface NodeScheduleResponse {
    schedule: { name: string } | null
    blocks: ScheduleBlock[]
    content: Content[]
}

interface EffectiveScheduleViewerProps {
    data: NodeScheduleResponse
    timezone: string
    onClose: () => void
}

export function EffectiveScheduleViewer({ data, timezone, onClose }: EffectiveScheduleViewerProps) {
    const scrollRef = useRef<HTMLDivElement>(null)

    // Helper: Convert UTC time string to target timezone time string (HH:MM)
    const formatTime = (timeStr: string, dateStr?: string) => {
        try {
            const utcDate = new Date(`${dateStr || '1970-01-01'}T${timeStr}Z`)
            return new Intl.DateTimeFormat('en-US', {
                hour: 'numeric',
                minute: 'numeric',
                hour12: true,
                timeZone: timezone
            }).format(utcDate)
        } catch (e) {
            return timeStr.substring(0, 5)
        }
    }

    // Determine "Now" to highlight active block
    // We need to compare current wall time in target timezone with block start times.
    // However, blocks are UTC. Easiest is to compare everything in UTC.
    // But the LIST should show Local Time.
    const now = new Date()
    // const nowUtcMinutes = now.getUTCHours() * 60 + now.getUTCMinutes()

    // Sort blocks by time just in case
    const sortedBlocks = useMemo(() => {
        return [...data.blocks].sort((a, b) => {
            // Construct full ISO strings for comparison
            const aDate = a.specific_date || '1970-01-01'
            const bDate = b.specific_date || '1970-01-01'

            if (aDate !== bDate) {
                return aDate.localeCompare(bDate)
            }
            return a.start_time.localeCompare(b.start_time)
        })
    }, [data.blocks])

    // Find active block index
    // We compare current UTC time with block UTC range
    const activeBlockIndex = useMemo(() => {
        const nowMs = now.getTime()

        return sortedBlocks.findIndex((block) => {
            const blockDateStr = block.specific_date || '1970-01-01'
            const startCmp = new Date(`${blockDateStr}T${block.start_time}Z`)
            const startMs = startCmp.getTime()
            const endMs = startMs + (block.duration_minutes * 60 * 1000)

            return nowMs >= startMs && nowMs < endMs
        })
    }, [sortedBlocks, now])


    // Auto-scroll to active block on mount
    useEffect(() => {
        if (activeBlockIndex !== -1 && scrollRef.current) {
            const row = scrollRef.current.children[activeBlockIndex] as HTMLElement
            if (row) {
                row.scrollIntoView({ behavior: 'smooth', block: 'center' })
            }
        }
    }, [activeBlockIndex])

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
            <div className="glass-panel flex flex-col w-full max-w-4xl max-h-[85vh] rounded-xl border border-[var(--border-color)] overflow-hidden shadow-2xl">

                {/* Header */}
                <div className="p-4 border-b border-[var(--border-color)] flex justify-between items-center bg-[var(--bg-secondary)]">
                    <div>
                        <h2 className="text-xl font-bold text-white">Effective Schedule</h2>
                        <p className="text-sm text-[var(--text-secondary)]">
                            Timezone: <span className="text-indigo-300">{timezone}</span>
                        </p>
                    </div>
                    <button onClick={onClose} className="p-2 hover:bg-white/10 rounded-lg transition-colors">
                        <svg className="w-5 h-5 text-[var(--text-secondary)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                {/* Table Header */}
                <div className="grid grid-cols-12 gap-4 p-3 bg-[var(--bg-tertiary)] border-b border-[var(--border-color)] text-xs font-medium text-[var(--text-secondary)] uppercase tracking-wider">
                    <div className="col-span-2">Time</div>
                    <div className="col-span-6">Content</div>
                </div>

                {/* Scrollable List */}
                <div className="flex-1 overflow-y-auto custom-scrollbar bg-[var(--bg-secondary)] relative">
                    <div ref={scrollRef} className="divide-y divide-[var(--border-color)]/50">
                        {sortedBlocks.length > 0 ? (
                            sortedBlocks.map((block, index) => {
                                const content = data.content.find(c => c.id === block.content_id)
                                const isActive = index === activeBlockIndex

                                return (
                                    <div
                                        key={block.id}
                                        className={`grid grid-cols-12 gap-4 p-3 items-center transition-colors ${isActive
                                            ? 'bg-indigo-500/10 border-l-4 border-indigo-500'
                                            : 'hover:bg-[var(--bg-primary)]/50 border-l-4 border-transparent'
                                            }`}
                                    >
                                        {/* Time Column */}
                                        <div className="col-span-2 flex flex-col">
                                            <span className={`font-mono font-medium ${isActive ? 'text-indigo-300' : 'text-white'}`}>
                                                {formatTime(block.start_time, block.specific_date)}
                                            </span>
                                            <span className="text-xs text-[var(--text-secondary)]">
                                                {block.duration_minutes}m
                                            </span>
                                        </div>

                                        {/* Content Column */}
                                        <div className="col-span-6 flex flex-col gap-1">
                                            <div className={`font-medium ${isActive ? 'text-white' : 'text-gray-300'}`}>
                                                {content?.title || `Unknown Content #${block.content_id}`}
                                            </div>
                                            {/* Source Indicator */}
                                            <div className="text-[10px] uppercase font-bold tracking-wider text-[var(--text-secondary)] flex items-center gap-1">
                                                <span className="w-1.5 h-1.5 rounded-full bg-indigo-500/50"></span>
                                                {block.source_schedule_name || 'Unknown Source'}
                                            </div>
                                        </div>

                                        {/* Status / Indicator */}
                                        <div className="col-span-4 flex justify-end">
                                            {isActive && (
                                                <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-indigo-500 text-white uppercase tracking-wider animate-pulse">
                                                    Now Playing
                                                </span>
                                            )}
                                        </div>
                                    </div>
                                )
                            })
                        ) : (
                            <div className="p-12 text-center text-[var(--text-secondary)] italic">
                                No schedule blocks found. Node will play fallback content.
                            </div>
                        )}
                    </div>
                </div>

                {/* Footer Actions */}
                <div className="p-4 border-t border-[var(--border-color)] bg-[var(--bg-secondary)] flex justify-end">
                    <button onClick={onClose} className="btn-secondary">Close</button>
                </div>
            </div>
        </div>
    )
}
