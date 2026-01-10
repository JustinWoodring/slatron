import { useEffect, useState } from 'react'
import { useDjStore } from '../../stores/djStore'
import { useAuthStore } from '../../stores/authStore'
import { AiProvider } from '../../api/dj'
import CreateAiProviderModal from '../Djs/CreateAiProviderModal'

export default function AiProviderList() {
    const { user } = useAuthStore()
    const { aiProviders, fetchAiProviders, removeAiProvider, isLoading, error } = useDjStore()

    const isEditor = user?.role === 'admin' || user?.role === 'editor'

    const [selectedProvider, setSelectedProvider] = useState<AiProvider | null>(null)
    const [isModalOpen, setIsModalOpen] = useState(false)
    const [toastMessage, setToastMessage] = useState<string | null>(null)

    useEffect(() => {
        fetchAiProviders()
    }, [fetchAiProviders])

    const handleEdit = (provider: AiProvider) => {
        setSelectedProvider(provider)
        setIsModalOpen(true)
    }

    const handleAdd = () => {
        setSelectedProvider(null)
        setIsModalOpen(true)
    }

    const handleDelete = async (id: number) => {
        if (!window.confirm("Are you sure you want to delete this provider?")) return
        try {
            await removeAiProvider(id)
            setToastMessage('Provider deleted successfully')
            setTimeout(() => setToastMessage(null), 3000)
        } catch (err) {
            console.error(err)
            alert('Failed to delete provider')
        }
    }

    const handleModalClose = () => {
        setIsModalOpen(false)
        setSelectedProvider(null)
        // Refresh list to show updates
        fetchAiProviders()
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-xl font-bold text-white">AI Providers</h2>
                    <p className="mt-1 text-sm text-[var(--text-secondary)]">Manage LLM & TTS Service Configurations</p>
                </div>
                {isEditor && (
                    <button
                        onClick={handleAdd}
                        className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg transition-colors shadow-lg shadow-indigo-500/20 flex items-center gap-2"
                    >
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                        </svg>
                        Add Provider
                    </button>
                )}
            </div>

            {error && (
                <div className="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400">
                    Error loading providers: {error}
                </div>
            )}

            {toastMessage && (
                <div className="fixed bottom-4 right-4 p-4 rounded-lg bg-green-500/20 border border-green-500/30 text-green-300 shadow-xl z-50 animate-fade-in">
                    {toastMessage}
                </div>
            )}

            {/* Table */}
            <div className="glass-panel overflow-hidden rounded-xl border border-[var(--border-color)]">
                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-[var(--bg-secondary)] border-b border-[var(--border-color)]">
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Name</th>
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Category</th>
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Type</th>
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Model/Voice</th>
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Status</th>
                                {isEditor && <th className="py-3 px-4 text-right text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Actions</th>}
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-[var(--border-color)]">
                            {aiProviders.map((p) => (
                                <tr key={p.id} className="hover:bg-[var(--bg-secondary)]/50 transition-colors">
                                    <td className="py-3 px-4 font-medium text-white">{p.name}</td>
                                    <td className="py-3 px-4">
                                        <span className={`text-xs px-2 py-1 rounded-full ${p.provider_category === 'llm' ? 'bg-blue-500/10 text-blue-400' : 'bg-purple-500/10 text-purple-400'}`}>
                                            {p.provider_category === 'llm' ? 'Text (LLM)' : 'Voice (TTS)'}
                                        </span>
                                    </td>
                                    <td className="py-3 px-4 text-[var(--text-secondary)]">{p.provider_type}</td>
                                    <td className="py-3 px-4 text-[var(--text-secondary)] font-mono text-sm">{p.model_name || '-'}</td>
                                    <td className="py-3 px-4">
                                        {p.is_active ?
                                            <span className="px-2 py-1 text-xs font-medium text-green-300 bg-green-500/10 rounded-full">Active</span> :
                                            <span className="px-2 py-1 text-xs font-medium text-gray-400 bg-gray-500/10 rounded-full">Inactive</span>
                                        }
                                    </td>
                                    {isEditor && (
                                        <td className="py-3 px-4 text-right">
                                            <button onClick={() => handleEdit(p)} className="text-indigo-400 hover:text-indigo-300 mr-3 text-sm font-medium">Edit</button>
                                            <button onClick={() => handleDelete(p.id)} className="text-red-400 hover:text-red-300 text-sm font-medium">Delete</button>
                                        </td>
                                    )}
                                </tr>
                            ))}
                            {aiProviders.length === 0 && !isLoading && (
                                <tr><td colSpan={6} className="py-8 text-center text-[var(--text-secondary)]">No providers found.</td></tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            <CreateAiProviderModal
                isOpen={isModalOpen}
                onClose={handleModalClose}
                initialData={selectedProvider}
            />
        </div>
    )
}
