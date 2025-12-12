import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import { useEffect, useRef } from 'react';

interface LogEntry {
    level: string;
    message: string;
    target: string;
    timestamp: string;
}

interface NodeLogsProps {
    nodeId: number;
}

export function NodeLogs({ nodeId }: NodeLogsProps) {
    const scrollRef = useRef<HTMLDivElement>(null);

    const { data: logs = [] } = useQuery<LogEntry[]>({
        queryKey: ['node-logs', nodeId],
        queryFn: async () => {
            const { data } = await apiClient.get(`/api/nodes/${nodeId}/logs`);
            return data;
        },
        refetchInterval: 2000,
    });

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [logs]);

    const getLevelColor = (level: string) => {
        switch (level.toLowerCase()) {
            case 'error': return 'text-red-400';
            case 'warn': return 'text-yellow-400';
            case 'info': return 'text-blue-400';
            case 'debug': return 'text-gray-400';
            default: return 'text-white';
        }
    };

    return (
        <div className="glass-panel rounded-lg p-4 h-[400px] flex flex-col">
            <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                <span>📜</span> Live Logs
            </h3>

            <div
                ref={scrollRef}
                className="flex-1 overflow-y-auto space-y-1 font-mono text-xs bg-black/30 rounded p-4"
            >
                {logs.length === 0 && (
                    <p className="text-gray-500 italic text-center mt-10">No logs received yet...</p>
                )}

                {logs.map((log, i) => (
                    <div key={i} className="flex gap-2 hover:bg-white/5 p-0.5 rounded">
                        <span className="text-gray-500 shrink-0">
                            {new Date(log.timestamp).toLocaleTimeString()}
                        </span>
                        <span className={`font-bold w-12 shrink-0 ${getLevelColor(log.level)}`}>
                            [{log.level}]
                        </span>
                        <span className="text-gray-400 w-24 shrink-0 truncate" title={log.target}>
                            {log.target}
                        </span>
                        <span className="text-gray-300 break-all">
                            {log.message}
                        </span>
                    </div>
                ))}
            </div>
        </div>
    );
}
