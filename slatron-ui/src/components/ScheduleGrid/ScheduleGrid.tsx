import React from 'react';
import {
    DragOverlay,
    useDroppable,
} from '@dnd-kit/core';
import { ScheduleBlock, DraggableScheduleBlock, BlockData } from './ScheduleBlock';
import { TimeAxis } from './TimeAxis';
import { useScheduleStore } from '../../stores/scheduleStore';
import { useContentStore } from '../../stores/contentStore';
import { snapCenterToCursor } from '@dnd-kit/modifiers'

// Helper to get minutes from midnight in target timezone
const getWallMinutesInTimezone = (date: Date, tz: string) => {
    try {
        const timeStr = date.toLocaleTimeString('en-US', { timeZone: tz, hour12: false });
        // Handle "24:00:00" edge case if it ever happens, though en-US usually 0-23
        const [h, m] = timeStr.split(':').map(Number);
        return (h % 24) * 60 + m;
    } catch {
        // Fallback to UTC
        return date.getUTCHours() * 60 + date.getUTCMinutes();
    }
}

interface ScheduleGridProps {
    pixelsPerMinute?: number;
    onGridClick?: (dayIndex: number, startTime: string, e: React.MouseEvent) => void;
    onBlockClick?: (blockId: number, e: React.MouseEvent) => void;
    activeId?: number | null;
    timezone?: string;
    readOnly?: boolean;
}

const DAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

// Separate component for DayColumn to use hooks
const DayColumn = ({
    dayIndex, blocks, pixelsPerMinute, onBlockClick, getTopOffset, handleColumnClick, readOnly
}: {
    dayIndex: number;
    blocks: BlockData[];
    pixelsPerMinute: number;
    onBlockClick?: (blockId: number, e: React.MouseEvent) => void;
    getTopOffset: (startTime: string) => number;
    handleColumnClick: (e: React.MouseEvent, dayIndex: number) => void;
    readOnly?: boolean;
}) => {
    const { setNodeRef } = useDroppable({
        id: `day-${dayIndex}`,
        disabled: readOnly
    });

    return (
        <div
            ref={setNodeRef}
            className={`flex-1 relative border-r border-[var(--border-color)]/30 last:border-r-0 h-full ${readOnly ? 'cursor-default' : 'cursor-cell'}`}
            onClick={(e) => !readOnly && handleColumnClick(e, dayIndex)}
        >
            <div className="relative w-full h-full">
                <div className="relative w-full h-full">
                    {blocks.map((block: BlockData) => (
                        <div
                            key={block.id}
                            className="absolute left-1 right-1 z-10"
                            style={{
                                top: getTopOffset(block.start_time),
                            }}
                            onClick={(e) => {
                                e.stopPropagation();
                                onBlockClick?.(block.id, e);
                            }}
                        >
                            {!readOnly ? (
                                <DraggableScheduleBlock
                                    block={{
                                        ...block,
                                        type: 'video', // Assuming a default type for display
                                        title: block.title || `Block ${block.id}`
                                    }}
                                    pixelsPerMinute={pixelsPerMinute}
                                />
                            ) : (
                                <ScheduleBlock
                                    block={{
                                        ...block,
                                        type: 'video',
                                        title: block.title || `Block ${block.id}`
                                    }}
                                    pixelsPerMinute={pixelsPerMinute}
                                />
                            )}
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export const ScheduleGrid = ({
    pixelsPerMinute = 2,
    onGridClick,
    onBlockClick,
    activeId,
    timezone = 'UTC',
    readOnly = false
}: ScheduleGridProps) => {
    const [now, setNow] = React.useState(new Date());

    React.useEffect(() => {
        const interval = setInterval(() => setNow(new Date()), 60000); // Update every minute
        return () => clearInterval(interval);
    }, []);

    const { blocks } = useScheduleStore();
    const { content } = useContentStore();

    // Enrich blocks with content titles
    const enrichedBlocks = React.useMemo(() => {
        return blocks.map(b => ({
            ...b,
            title: content.find(c => c.id === b.content_id)?.title || (b.content_id ? `Content #${b.content_id}` : 'Untitled Event'),
            type: 'video' // Default
        } as BlockData));
    }, [blocks, content]);

    const gridHeight = 24 * 60 * pixelsPerMinute;

    const getTopOffset = (timeStr: string) => {
        const [hours, minutes] = timeStr.split(':').map(Number);
        return (hours * 60 + minutes) * pixelsPerMinute;
    };

    const currentMinutes = getWallMinutesInTimezone(now, timezone);

    const handleColumnClick = (e: React.MouseEvent, dayIndex: number) => {
        if (!onGridClick || readOnly) return;

        // Get click position relative to the column
        // const rect = e.currentTarget.getBoundingClientRect();
        // Use nativeEvent.offsetY if possible for simplicity, or calc relative to top
        // Since the column can be scrolled (inside parent), e.nativeEvent.offsetY is usually local to element
        const y = e.nativeEvent.offsetY;

        const minutes = Math.floor(y / pixelsPerMinute);
        const hours = Math.floor(minutes / 60);
        const mins = minutes % 60;
        const snappedMins = Math.floor(mins / 15) * 15;

        const timeString = `${hours.toString().padStart(2, '0')}:${snappedMins.toString().padStart(2, '0')}:00`;
        onGridClick(dayIndex, timeString, e);
    };

    return (
        <div className="flex flex-col h-full rounded-xl overflow-hidden glass-panel border border-[var(--border-color)] relative">
            {/* Scrollable Grid Area */}
            <div className="flex-1 overflow-y-auto overflow-x-hidden relative custom-scrollbar">
                {/* Header (Sticky) */}
                <div className="sticky top-0 z-30 flex border-b border-[var(--border-color)] bg-[var(--bg-secondary)]">
                    <div className="w-16 flex-shrink-0 border-r border-[var(--border-color)] p-4 flex items-center justify-center bg-[var(--bg-tertiary)]">
                        <span className="text-xs text-[var(--text-secondary)] font-medium">Time</span>
                    </div>
                    <div className="flex-1 flex">
                        {DAYS.map(day => (
                            <div key={day} className="flex-1 p-2 text-center border-r border-[var(--border-color)] last:border-r-0">
                                <span className="text-sm font-bold text-white tracking-wider">{day}</span>
                            </div>
                        ))}
                    </div>
                </div>

                <div className="flex min-h-full">
                    {/* Time Axis */}
                    <div className="w-16 flex-shrink-0 bg-[var(--bg-primary)] border-r border-[var(--border-color)] z-10 relative">
                        <TimeAxis pixelsPerMinute={pixelsPerMinute} />
                    </div>

                    {/* Days Columns */}
                    <div id="schedule-grid-container" className="flex-1 relative bg-[var(--bg-primary)]/50 flex" style={{ height: gridHeight }}>
                        {/* Background Grid Lines (Horizontal) */}
                        <div className="absolute inset-0 pointer-events-none z-0">
                            {Array.from({ length: 24 }).map((_, i) => (
                                <div
                                    key={i}
                                    className="absolute w-full border-t border-[var(--border-color)]/30"
                                    style={{ top: i * 60 * pixelsPerMinute }}
                                />
                            ))}
                        </div>

                        {/* Current Time Ticker (Wall Time) */}
                        <div className="absolute w-full pointer-events-none z-20 border-t-2 border-red-500" style={{ top: currentMinutes * pixelsPerMinute }}>
                            <span className="absolute -top-3 left-0 bg-red-500 text-white text-[10px] px-1 rounded-r shadow-sm font-mono">
                                {timezone.split('/').pop()?.replace(/_/g, ' ') || 'Local'}
                            </span>
                        </div>

                        {/* 7 Columns */}
                        {Array.from({ length: 7 }).map((_, dayIndex) => (
                            <DayColumn
                                key={dayIndex}
                                dayIndex={dayIndex}
                                blocks={enrichedBlocks.filter(b => b.day_of_week === dayIndex)}
                                pixelsPerMinute={pixelsPerMinute}
                                onBlockClick={onBlockClick}
                                getTopOffset={getTopOffset}
                                handleColumnClick={handleColumnClick}
                            />
                        ))}

                        <DragOverlay modifiers={[snapCenterToCursor]}>
                            {activeId ? (
                                (() => {
                                    const block = enrichedBlocks.find(b => b.id === activeId);
                                    if (!block) return null;
                                    return (
                                        <div className="opacity-90 cursor-grabbing shadow-xl scale-105 pointer-events-none">
                                            <ScheduleBlock
                                                block={{
                                                    ...block,
                                                    type: 'video',
                                                    title: block.title || `Block ${block.id}`
                                                }}
                                                pixelsPerMinute={pixelsPerMinute}
                                                className="relative w-40 shadow-2xl" // Override absolute/w-full. w-40 is approx column width.
                                            />
                                        </div>
                                    )
                                })()
                            ) : null}
                        </DragOverlay>
                    </div>
                </div>
            </div>
        </div>
    );
};
