import React from 'react';


import clsx from 'clsx';

export interface BlockData {
    id: number;
    content_id: number;
    title: string;
    start_time: string; // HH:MM:SS
    duration_minutes: number;
    type: 'video' | 'stream' | 'image' | 'playlist';
    color?: string;
    day_of_week?: number | null;
    specific_date?: string | null;
    script_id?: number | null;
    schedule_id?: number;
    dj_id?: number | null;
    dj_name?: string | null;
}

interface ScheduleBlockProps {
    block: BlockData;
    pixelsPerMinute: number;
    showDetails?: boolean;
    // Dnd props (optional for static rendering)
    isDragging?: boolean;
    style?: React.CSSProperties;
    attributes?: any;
    listeners?: any;
    setNodeRef?: (node: HTMLElement | null) => void;
}

// Visual Component (No Hooks)
export const ScheduleBlock = ({
    block,
    pixelsPerMinute,
    showDetails = true,
    isDragging,
    style,
    attributes,
    listeners,
    setNodeRef,
    className
}: ScheduleBlockProps & { className?: string }) => {

    const getBackgroundColor = (type: string) => {
        // Special styling for DJ blocks
        if (block.dj_id) {
            return 'bg-orange-600/30 border-orange-500/60 hover:bg-orange-600/40 shadow-sm shadow-orange-900/20';
        }

        switch (type) {
            case 'video': return 'bg-cyan-600/20 border-cyan-500/50 hover:bg-cyan-600/30';
            case 'stream': return 'bg-purple-600/20 border-purple-500/50 hover:bg-purple-600/30';
            case 'image': return 'bg-amber-600/20 border-amber-500/50 hover:bg-amber-600/30';
            case 'live': return 'bg-rose-600/20 border-rose-500/50 hover:bg-rose-600/30';
            default: return 'bg-indigo-600/20 border-indigo-500/50 hover:bg-indigo-600/30';
        }
    };

    const getTextColor = (type: string) => {
        if (block.dj_id) {
            return 'text-orange-200';
        }

        switch (type) {
            case 'video': return 'text-cyan-200';
            case 'stream': return 'text-purple-200';
            case 'image': return 'text-amber-200';
            case 'live': return 'text-rose-200';
            default: return 'text-indigo-200';
        }
    };

    return (
        <div
            ref={setNodeRef}
            style={{ ...style, height: `${block.duration_minutes * pixelsPerMinute}px` }}
            {...attributes}
            {...listeners}
            className={clsx(
                className || 'absolute w-full',
                'rounded-md border-l-4 p-2 text-xs overflow-hidden cursor-move transition-colors group',
                getBackgroundColor(block.type),
                isDragging ? 'opacity-50 z-50 ring-2 ring-white' : 'z-10',
                'backdrop-blur-sm'
            )}
        >
            <div className="flex justify-between items-start">
                <span className={clsx("font-semibold truncate", getTextColor(block.type))}>
                    {block.title}
                </span>
                {showDetails && (
                    <span className="text-[10px] opacity-70 ml-1">
                        {block.duration_minutes}m
                    </span>
                )}
            </div>

            {showDetails && block.duration_minutes > 15 && (
                <div className="mt-1 text-[10px] opacity-60 truncate">
                    {block.start_time}
                </div>
            )}

            {/* Hover Actions */}
            <div className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                <button className="p-1 hover:bg-black/20 rounded">
                    <svg className="w-3 h-3 text-white/70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                    </svg>
                </button>
            </div>

            {/* Resize Handle (Visual Only for now) */}
            <div className="absolute bottom-0 left-0 right-0 h-2 cursor-ns-resize hover:bg-white/10" />
        </div>
    );
};

// Sortable Wrapper (With Hooks)
import { useDraggable } from '@dnd-kit/core';

export const DraggableScheduleBlock = (props: ScheduleBlockProps) => {
    const {
        attributes,
        listeners,
        setNodeRef,
        isDragging,
    } = useDraggable({ id: props.block.id });

    // We do NOT want to move the original element if we are using an Overlay.
    // The original should just fade out to show it's being moved.
    // So we pass undefined for style transform.
    const style: React.CSSProperties = {
        opacity: isDragging ? 0.3 : 1,
    };

    return (
        <ScheduleBlock
            {...props}
            setNodeRef={setNodeRef}
            style={style}
            attributes={attributes}
            listeners={listeners}
            isDragging={isDragging}
        />
    );
};
