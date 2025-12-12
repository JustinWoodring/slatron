// import React from 'react';
// import clsx from 'clsx';

interface TimeAxisProps {
    pixelsPerMinute: number;
}

export const TimeAxis = ({ pixelsPerMinute }: TimeAxisProps) => {
    const hours = Array.from({ length: 24 }, (_, i) => i);
    const hourHeight = 60 * pixelsPerMinute;

    return (
        <div className="flex-shrink-0 w-18 bg-[var(--bg-secondary)] border-r border-[var(--border-color)]">
            <div className="relative" style={{ height: 24 * 60 * pixelsPerMinute }}>
                {hours.map((hour) => (
                    <div
                        key={hour}
                        className="absolute w-full px-2 text-right border-t border-[var(--border-color)]/30"
                        style={{
                            top: hour * 60 * pixelsPerMinute,
                            height: hourHeight,
                        }}
                    >
                        <span className="text-xs text-[var(--text-secondary)] -mt-2.5 inline-block bg-[var(--bg-secondary)] pl-1">
                            {hour.toString().padStart(2, '0')}:00
                        </span>
                        {/* 30 min marker */}
                        <div
                            className="absolute w-2 right-0 border-t border-[var(--border-color)]/20"
                            style={{ top: '50%' }}
                        />
                    </div>
                ))}
                {/* Current Time Indicator logic could go here */}
            </div>
        </div>
    );
};
