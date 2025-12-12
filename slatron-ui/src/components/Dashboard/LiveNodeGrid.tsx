import { formatDistanceToNow } from 'date-fns'

interface Node {
    id: number
    name: string
    status: string
    current_content_id: number | null
    playback_position_secs: number | null
    last_heartbeat: string | null
}

interface ContentItem {
    id: number
    title: string
    duration_minutes: number
}

interface LiveNodeGridProps {
    nodes: Node[]
    content: ContentItem[]
}

export function LiveNodeGrid({ nodes, content }: LiveNodeGridProps) {
    const getStatusColor = (status: string) => {
        switch (status) {
            case 'online': return 'bg-green-500 shadow-[0_0_15px_rgba(34,197,94,0.6)]'
            case 'offline': return 'bg-gray-500'
            case 'error': return 'bg-red-500 shadow-[0_0_15px_rgba(239,68,68,0.6)]'
            default: return 'bg-gray-500'
        }
    }

    const getContentDetails = (contentId: number | null) => {
        if (!contentId) return null
        return content.find(c => c.id === contentId)
    }

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
            {nodes.map(node => {
                const currentContent = getContentDetails(node.current_content_id)
                const progress = currentContent && node.playback_position_secs && currentContent.duration_minutes
                    ? (node.playback_position_secs / (currentContent.duration_minutes * 60)) * 100
                    : 0

                return (
                    <div key={node.id} className="glass-panel relative overflow-hidden group hover:border-indigo-500/50 transition-colors">
                        {/* Screen Bezel Effect */}
                        <div className="aspect-video bg-black/80 rounded-t-xl relative overflow-hidden border-b border-[var(--border-color)]">
                            {/* Active Content Preview (Abstract) */}
                            {node.status === 'online' && currentContent ? (
                                <>
                                    <div className="absolute inset-0 flex items-center justify-center">
                                        {/* Placeholder Animation */}
                                        <div className="w-full h-full bg-gradient-to-br from-indigo-900/40 to-black animate-pulse flex items-center justify-center">
                                            <span className="text-4xl">📺</span>
                                        </div>
                                    </div>
                                    {/* Progress Bar overlay */}
                                    <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/10">
                                        <div
                                            className="h-full bg-indigo-500 transition-all duration-1000 ease-linear shadow-[0_0_10px_rgba(99,102,241,0.5)]"
                                            style={{ width: `${Math.min(100, progress)}%` }}
                                        />
                                    </div>
                                    {/* Status Indicator */}
                                    <div className="absolute top-3 right-3 flex items-center gap-2 bg-black/60 backdrop-blur-md px-2 py-1 rounded-full border border-white/10">
                                        <div className={`w-2 h-2 rounded-full ${getStatusColor(node.status)} animate-pulse`} />
                                        <span className="text-[10px] font-bold text-white uppercase tracking-wider">Live</span>
                                    </div>
                                </>
                            ) : (
                                <div className="absolute inset-0 flex flex-col items-center justify-center text-[var(--text-secondary)] bg-[var(--bg-secondary)]/50">
                                    <div className={`w-3 h-3 rounded-full mb-2 ${getStatusColor(node.status)}`} />
                                    <span className="text-xs font-mono uppercase tracking-widest">{node.status}</span>
                                </div>
                            )}
                        </div>

                        {/* Info Panel */}
                        <div className="p-4 bg-[var(--bg-secondary)]/30">
                            <div className="flex justify-between items-start mb-2">
                                <div>
                                    <h3 className="text-lg font-bold text-white group-hover:text-indigo-300 transition-colors">{node.name}</h3>
                                    <p className="text-xs text-[var(--text-secondary)] font-mono mt-0.5">ID: {node.id.toString().padStart(4, '0')}</p>
                                </div>
                                {currentContent && (
                                    <div className="text-right">
                                        <div className="text-xs text-indigo-300 font-medium bg-indigo-500/10 px-2 py-0.5 rounded border border-indigo-500/20">
                                            Now Playing
                                        </div>
                                    </div>
                                )}
                            </div>

                            {currentContent ? (
                                <div className="mt-3">
                                    <p className="text-sm text-white font-medium truncate">{currentContent.title}</p>
                                    <p className="text-xs text-[var(--text-secondary)] mt-1 flex justify-between">
                                        <span>{Math.floor((node.playback_position_secs || 0) / 60)}:{(Math.floor((node.playback_position_secs || 0) % 60)).toString().padStart(2, '0')}</span>
                                        <span>{currentContent.duration_minutes}m</span>
                                    </p>
                                </div>
                            ) : (
                                <div className="mt-3 h-10 flex items-center text-xs text-[var(--text-secondary)] italic">
                                    No content scheduled
                                </div>
                            )}

                            <div className="mt-4 pt-3 border-t border-white/5 flex justify-between items-center text-[10px] text-[var(--text-secondary)] uppercase tracking-wider">
                                <span>Last Seen: {node.last_heartbeat ? formatDistanceToNow(new Date(node.last_heartbeat)) + ' ago' : 'Never'}</span>
                            </div>
                        </div>
                    </div>
                )
            })}
        </div>
    )
}
