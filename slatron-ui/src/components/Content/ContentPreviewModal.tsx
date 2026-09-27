import { Link } from 'react-router-dom'
import type { ContentItem } from '../../hooks/useContent'

interface ContentPreviewModalProps {
    item?: ContentItem
    onClose: () => void
}

function typeBadgeClass(type: string) {
    if (type === 'local_file') return 'bg-emerald-500/10 text-emerald-400'
    if (type === 'remote_url') return 'bg-blue-500/10 text-blue-400'
    if (type === 'spot_reel') return 'bg-violet-500/10 text-violet-400'
    return 'bg-amber-500/10 text-amber-400'
}

function NodeFileNotice({ path }: { path: string }) {
    return (
        <div className="flex flex-col items-center justify-center p-8 border border-[var(--border-color)] rounded-lg bg-[var(--bg-primary)]/50 text-center">
            <svg className="w-10 h-10 mb-3 text-[var(--text-secondary)] opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
            </svg>
            <p className="text-sm text-[var(--text-secondary)]">Preview unavailable — file lives on the playback node</p>
            <code className="mt-2 text-xs font-mono text-[var(--text-secondary)] break-all">{path}</code>
        </div>
    )
}

function ExternalLinkNotice({ path }: { path: string }) {
    return (
        <div className="flex flex-col items-center justify-center p-8 border border-[var(--border-color)] rounded-lg bg-[var(--bg-primary)]/50 text-center">
            <svg className="w-10 h-10 mb-3 text-[var(--text-secondary)] opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
            <p className="text-sm text-[var(--text-secondary)]">Web and external content can't be previewed inline.</p>
            <a
                href={path}
                target="_blank"
                rel="noopener noreferrer"
                className="mt-2 text-sm text-indigo-400 hover:text-indigo-300 underline break-all"
            >
                {path}
            </a>
            <p className="mt-1 text-xs text-[var(--text-secondary)] opacity-70">Opens in a new tab</p>
        </div>
    )
}

function PreviewBody({ item }: { item: ContentItem }) {
    const type = item.content_type
    const previewable = item.content_path.startsWith('http') || item.content_path.startsWith('/')

    if (type === 'spot_reel') {
        return (
            <div className="flex flex-col items-center justify-center p-8 border border-[var(--border-color)] rounded-lg bg-[var(--bg-primary)]/50 text-center">
                <svg className="w-10 h-10 mb-3 text-violet-400/70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 4v3m8-3v3m-9 8h10M5 21h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2zM6 2v4m12-4v4" />
                </svg>
                <p className="text-sm text-white">This item is a spot reel — a looping carousel of images, videos, and web pages.</p>
                <p className="mt-1 text-xs text-[var(--text-secondary)]">It plays on nodes as a continuous loop; preview its items from the Spot Reels editor.</p>
                <Link to="/spot-reels" className="mt-4 text-sm text-violet-400 hover:text-violet-300 underline">
                    Open Spot Reels
                </Link>
            </div>
        )
    }

    if (type.includes('video')) {
        return previewable ? (
            <video
                key={item.id}
                src={item.content_path}
                controls
                autoPlay={false}
                className="w-full max-h-[60vh] rounded-lg bg-black"
            />
        ) : (
            <NodeFileNotice path={item.content_path} />
        )
    }

    if (type.includes('image')) {
        return previewable ? (
            <img
                key={item.id}
                src={item.content_path}
                alt={item.title}
                className="w-full max-h-[60vh] rounded-lg object-contain bg-black/40"
            />
        ) : (
            <NodeFileNotice path={item.content_path} />
        )
    }

    return previewable ? (
        <ExternalLinkNotice path={item.content_path} />
    ) : (
        <NodeFileNotice path={item.content_path} />
    )
}

export default function ContentPreviewModal({ item, onClose }: ContentPreviewModalProps) {
    if (!item) return null

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl w-full max-w-2xl shadow-2xl animate-fade-in flex flex-col max-h-[90vh]">

                {/* Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-[var(--border-color)]">
                    <div className="min-w-0">
                        <div className="flex items-center gap-2">
                            <span className={`text-[10px] uppercase tracking-wider font-bold px-2 py-1 rounded-full ${typeBadgeClass(item.content_type)}`}>
                                {item.content_type.replace('_', ' ')}
                            </span>
                        </div>
                        <h3 className="mt-1 text-white font-medium truncate" title={item.title}>{item.title}</h3>
                    </div>
                    <button onClick={onClose} className="text-[var(--text-secondary)] hover:text-white transition-colors" title="Close">
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                {/* Body */}
                <div className="p-6 overflow-y-auto custom-scrollbar">
                    <PreviewBody item={item} />
                    {item.description && (
                        <p className="mt-3 text-sm text-[var(--text-secondary)]">{item.description}</p>
                    )}
                </div>
            </div>
        </div>
    )
}
