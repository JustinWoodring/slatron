import { useState, useEffect } from 'react'
import { useDjStore } from '../stores/djStore'
import CreateDjModal from '../components/Djs/CreateDjModal'
import AiProviderList from '../components/Settings/AiProviderList'

export default function DjsPage() {
    const { djs, fetchDjs, removeDj, isLoading } = useDjStore()
    const [activeTab, setActiveTab] = useState<'djs' | 'ai'>('djs')

    // Modals
    const [isDjModalOpen, setIsDjModalOpen] = useState(false)
    const [selectedDj, setSelectedDj] = useState<any>(null) // Using any for simplicity with imports, or proper type

    useEffect(() => {
        fetchDjs()
    }, [])

    const handleCreateDj = () => {
        setSelectedDj(null)
        setIsDjModalOpen(true)
        setActiveTab('djs')
    }

    const handleEditDj = (dj: any) => {
        setSelectedDj(dj)
        setIsDjModalOpen(true)
    }

    return (
        <div className="h-full flex flex-col">
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h1 className="text-2xl font-bold text-white mb-2">DJs & AI Integration</h1>
                    <p className="text-[var(--text-secondary)]">Manage virtual personalities and AI service connections</p>
                </div>
                <button
                    onClick={handleCreateDj}
                    className="btn-primary flex items-center gap-2"
                    style={{ visibility: activeTab === 'djs' ? 'visible' : 'hidden' }}
                >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                    </svg>
                    {activeTab === 'djs' ? 'Create DJ' : 'Add AI Provider'}
                </button>
            </div>

            {/* Tabs */}
            <div className="flex border-b border-[var(--border-color)] mb-6">
                <button
                    onClick={() => setActiveTab('djs')}
                    className={`px-6 py-3 text-sm font-medium transition-colors ${activeTab === 'djs' ? 'text-emerald-400 border-b-2 border-emerald-400' : 'text-[var(--text-secondary)] hover:text-white'}`}
                >
                    DJ Profiles
                </button>
                <button
                    onClick={() => setActiveTab('ai')}
                    className={`px-6 py-3 text-sm font-medium transition-colors ${activeTab === 'ai' ? 'text-emerald-400 border-b-2 border-emerald-400' : 'text-[var(--text-secondary)] hover:text-white'}`}
                >
                    AI Providers
                </button>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto custom-scrollbar">
                {isLoading && djs.length === 0 ? (
                    <div className="flex items-center justify-center h-64">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white"></div>
                    </div>
                ) : (
                    activeTab === 'djs' ? (
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            {djs.map(dj => (
                                <div key={dj.id} className="glass-panel p-6 rounded-xl border border-[var(--border-color)] hover:border-[var(--accent-primary)]/50 transition-colors group">
                                    <div className="flex justify-between items-start mb-4">
                                        <div className="w-12 h-12 rounded-full bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center text-white font-bold text-lg">
                                            {dj.name.charAt(0).toUpperCase()}
                                        </div>
                                        <button
                                            onClick={() => removeDj(dj.id)}
                                            className="text-[var(--text-secondary)] hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
                                        >
                                            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                            </svg>
                                        </button>
                                    </div>
                                    <h3 className="text-lg font-bold text-white mb-2">{dj.name}</h3>
                                    <p className="text-sm text-[var(--text-secondary)] line-clamp-3 mb-4 h-15">
                                        {dj.personality_prompt || "No personality defined."}
                                    </p>
                                    <div className="pt-4 border-t border-[var(--border-color)] flex justify-between items-center">
                                        <span className="text-xs text-[var(--text-secondary)]">
                                            Voice: {(() => {
                                                try { return JSON.parse(dj.voice_config_json || '{}').voice_name || 'Default' }
                                                catch { return 'Default' }
                                            })()}
                                        </span>
                                        <button
                                            onClick={() => handleEditDj(dj)}
                                            className="text-xs text-indigo-400 hover:text-indigo-300 font-medium"
                                        >
                                            Edit
                                        </button>
                                    </div>
                                </div>
                            ))}
                            {djs.length === 0 && (
                                <div className="col-span-full text-center py-12 text-[var(--text-secondary)]">
                                    No DJ profiles found. Create one to get started.
                                </div>
                            )}
                        </div>
                    ) : (
                        <AiProviderList />
                    )
                )}
            </div>

            <CreateDjModal
                isOpen={isDjModalOpen}
                onClose={() => setIsDjModalOpen(false)}
                onDjAdded={fetchDjs}
                initialDj={selectedDj}
            />
        </div>
    )
}
