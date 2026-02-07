import { useState, useEffect } from 'react'
import { ContentItem } from '../../stores/contentStore'

interface ContentPickerModalProps {
    isOpen: boolean
    onClose: () => void
    onSelect: (contentId: number) => void
    content: ContentItem[]
    currentId: number | string | null
}

export const ContentPickerModal = ({ isOpen, onClose, onSelect, content, currentId }: ContentPickerModalProps) => {
    const [search, setSearch] = useState('')
    const [filteredContent, setFilteredContent] = useState<ContentItem[]>(content)

    useEffect(() => {
        if (!search) {
            setFilteredContent(content)
        } else {
            const lowerSearch = search.toLowerCase()
            setFilteredContent(content.filter(item =>
                item.title.toLowerCase().includes(lowerSearch) ||
                item.content_type.toLowerCase().includes(lowerSearch)
            ))
        }
    }, [search, content])

    if (!isOpen) return null

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-fade-in">
            <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-2xl w-full max-w-lg flex flex-col max-h-[80vh]">

                {/* Header */}
                <div className="p-4 border-b border-[var(--border-color)] flex justify-between items-center">
                    <h3 className="text-lg font-bold text-white">Select Content</h3>
                    <button onClick={onClose} className="p-1 hover:bg-[var(--bg-tertiary)] rounded-full text-[var(--text-secondary)] hover:text-white transition-colors">
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                {/* Search */}
                <div className="p-4 border-b border-[var(--border-color)] bg-[var(--bg-tertiary)]/30">
                    <div className="relative">
                        <svg className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--text-secondary)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                        <input
                            type="text"
                            placeholder="Search content..."
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg pl-9 pr-4 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            autoFocus
                        />
                    </div>
                </div>

                {/* List */}
                <div className="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
                    <div
                        onClick={() => onSelect(0)} // 0 or null for "No Content"
                        className={`
                            p-3 rounded-lg flex items-center justify-between cursor-pointer transition-colors
                            ${currentId === '' || currentId === null || currentId === 0 ? 'bg-indigo-500/20 border border-indigo-500/50' : 'hover:bg-[var(--bg-tertiary)] border border-transparent'}
                        `}
                    >
                        <span className="text-sm font-medium text-[var(--text-secondary)]">No Content (Placeholder)</span>
                        {(currentId === '' || currentId === null || currentId === 0) && (
                            <svg className="w-4 h-4 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                            </svg>
                        )}
                    </div>

                    {filteredContent.map(item => (
                        <div
                            key={item.id}
                            onClick={() => onSelect(item.id)}
                            className={`
                                p-3 rounded-lg flex items-center justify-between cursor-pointer transition-colors group
                                ${currentId == item.id ? 'bg-indigo-500/20 border border-indigo-500/50' : 'hover:bg-[var(--bg-tertiary)] border border-transparent'}
                            `}
                        >
                            <div className="flex flex-col">
                                <span className={`text-sm font-medium ${currentId == item.id ? 'text-white' : 'text-gray-300 group-hover:text-white'}`}>
                                    {item.title}
                                </span>
                                <div className="flex items-center gap-2 mt-1">
                                    <span className={`text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded border ${
                                        item.content_type === 'spot_reel'
                                            ? 'bg-purple-500/20 text-purple-300 border-purple-500/30'
                                            : 'bg-[var(--bg-primary)] text-[var(--text-secondary)] border-[var(--border-color)]'
                                    }`}>
                                        {item.content_type === 'spot_reel' ? 'Spot Reel' : item.content_type}
                                    </span>
                                    {item.duration_minutes && (
                                        <span className="text-xs text-[var(--text-tertiary)]">
                                            {item.duration_minutes}m
                                        </span>
                                    )}
                                </div>
                            </div>

                            {currentId == item.id && (
                                <svg className="w-4 h-4 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                </svg>
                            )}
                        </div>
                    ))}

                    {filteredContent.length === 0 && (
                        <div className="p-8 text-center text-[var(--text-secondary)] text-sm">
                            No content found matching "{search}"
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="p-4 border-t border-[var(--border-color)] flex justify-end">
                    <button
                        onClick={onClose}
                        className="px-4 py-2 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-white transition-colors text-sm"
                    >
                        Cancel
                    </button>
                </div>
            </div>
        </div>
    )
}
