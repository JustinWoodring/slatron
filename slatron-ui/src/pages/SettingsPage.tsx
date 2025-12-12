import { useEffect, useState } from 'react'
import { apiClient } from '../api/client'
import { useAuthStore } from '../stores/authStore'


interface GlobalSetting {
    id?: number
    key: string
    value: string
    description?: string
    updated_at: string
}

export default function SettingsPage() {
    const { user } = useAuthStore()
    const isEditor = user?.role === 'admin' || user?.role === 'editor'

    const [settings, setSettings] = useState<GlobalSetting[]>([])
    const [isLoading, setIsLoading] = useState(true)
    const [selectedSetting, setSelectedSetting] = useState<GlobalSetting | null>(null)
    const [editValue, setEditValue] = useState('')
    const [isModalOpen, setIsModalOpen] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [toastMessage, setToastMessage] = useState<string | null>(null)



    // Auto-hide toast
    useEffect(() => {
        if (toastMessage) {
            const timer = setTimeout(() => setToastMessage(null), 3000)
            return () => clearTimeout(timer)
        }
    }, [toastMessage])

    const fetchSettings = async () => {
        setIsLoading(true)
        setError(null)
        try {
            const { data } = await apiClient.get('/api/settings')
            setSettings(data)
        } catch (err) {
            setError(String(err))
        } finally {
            setIsLoading(false)
        }
    }

    useEffect(() => {
        fetchSettings()
    }, [])

    const handleEdit = (setting: GlobalSetting) => {
        setSelectedSetting(setting)
        setEditValue(setting.value)
        setIsModalOpen(true)
    }

    const handleCloseModal = () => {
        setIsModalOpen(false)
        setSelectedSetting(null)
    }

    const handleAdd = () => {
        setSelectedSetting({ key: '', value: '', description: '' } as GlobalSetting)
        setEditValue('')
        setIsModalOpen(true)
    }

    const handleSave = async () => {
        if (!selectedSetting) return

        // Use key from input for new, or existing key for edit
        const keyToSave = selectedSetting.key || (document.getElementById('setting-key') as HTMLInputElement)?.value

        if (!keyToSave) {
            setError("Key is required")
            return
        }

        try {
            await apiClient.put(`/api/settings/${keyToSave}`, {
                key: keyToSave,
                value: editValue,
                description: selectedSetting.description || "Manually added setting",
            })

            setToastMessage('Setting updated successfully')
            handleCloseModal()
            fetchSettings()
        } catch (err) {
            setError(String(err))
        }
    }

    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-full">
                <div className="text-[var(--text-secondary)]">Loading settings...</div>
            </div>
        )
    }

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-white">Global Settings</h1>
                    <p className="mt-1 text-sm text-[var(--text-secondary)]">Manage server configuration</p>
                </div>
                {isEditor && (
                    <button
                        onClick={handleAdd}
                        className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg transition-colors shadow-lg shadow-indigo-500/20 flex items-center gap-2"
                    >
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                        </svg>
                        Add Setting
                    </button>
                )}
            </div>

            {error && (
                <div className="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400">
                    {error}
                </div>
            )}

            {toastMessage && (
                <div className="fixed bottom-4 right-4 p-4 rounded-lg bg-green-500/20 border border-green-500/30 text-green-300 shadow-xl z-50">
                    {toastMessage}
                </div>
            )}

            {/* Table */}
            <div className="glass-panel overflow-hidden rounded-xl border border-[var(--border-color)]">
                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-[var(--bg-secondary)] border-b border-[var(--border-color)]">
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Key</th>
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Value</th>
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Description</th>
                                <th className="py-3 px-4 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Last Updated</th>
                                {isEditor && (
                                    <th className="py-3 px-4 text-right text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Actions</th>
                                )}
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-[var(--border-color)]">
                            {settings.map((s) => (
                                <tr key={s.key} className="hover:bg-[var(--bg-secondary)]/50 transition-colors">
                                    <td className="py-3 px-4 font-medium text-white font-mono text-sm">{s.key}</td>
                                    <td className="py-3 px-4 text-[var(--text-primary)] font-mono text-sm max-w-xs truncate" title={s.value}>{s.value}</td>
                                    <td className="py-3 px-4 text-[var(--text-secondary)] text-sm">{s.description}</td>
                                    <td className="py-3 px-4 text-[var(--text-secondary)] text-sm whitespace-nowrap">
                                        {new Date(s.updated_at).toLocaleString()}
                                    </td>
                                    {isEditor && (
                                        <td className="py-3 px-4 text-right">
                                            <button
                                                onClick={() => handleEdit(s)}
                                                className="px-3 py-1 text-sm font-medium text-indigo-300 hover:text-white bg-indigo-500/10 hover:bg-indigo-500/20 rounded-md transition-colors"
                                            >
                                                Edit
                                            </button>
                                        </td>
                                    )}
                                </tr>
                            ))}
                            {settings.length === 0 && (
                                <tr>
                                    <td colSpan={5} className="py-8 text-center text-[var(--text-secondary)]">
                                        No settings found.
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* Edit Modal */}
            {isModalOpen && selectedSetting && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
                    <div className="w-full max-w-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-2xl overflow-hidden glass-panel">
                        <div className="px-6 py-4 border-b border-[var(--border-color)] flex justify-between items-center">
                            <h3 className="text-lg font-bold text-white">
                                {selectedSetting.id ? 'Edit Setting' : 'Add New Setting'}
                            </h3>
                            <button onClick={handleCloseModal} className="text-[var(--text-secondary)] hover:text-white">
                                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>

                        <div className="p-6 space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Key</label>
                                {selectedSetting.id ? (
                                    <div className="px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white font-mono opacity-75">
                                        {selectedSetting.key}
                                    </div>
                                ) : (
                                    <input
                                        id="setting-key"
                                        type="text"
                                        defaultValue={selectedSetting.key}
                                        onChange={(e) => selectedSetting.key = e.target.value} // Mutable ref approach for new item
                                        className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                                        placeholder="e.g. timezone"
                                    />
                                )}
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Value</label>
                                {selectedSetting.key === 'timezone' || (!selectedSetting.id && (document.getElementById('setting-key') as HTMLInputElement)?.value === 'timezone') ? (
                                    <select
                                        value={editValue}
                                        onChange={(e) => setEditValue(e.target.value)}
                                        className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50 appearance-none"
                                    >
                                        <option value="">Select Timezone...</option>
                                        {(Intl as any).supportedValuesOf('timeZone').map((tz: string) => (
                                            <option key={tz} value={tz}>{tz}</option>
                                        ))}
                                    </select>
                                ) : (
                                    <input
                                        type="text"
                                        value={editValue}
                                        onChange={(e) => setEditValue(e.target.value)}
                                        className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                                        placeholder="Enter value"
                                    />
                                )}
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Description</label>
                                <input
                                    type="text"
                                    defaultValue={selectedSetting.description}
                                    onChange={(e) => selectedSetting.description = e.target.value}
                                    className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                                    placeholder="Optional description"
                                />
                            </div>
                        </div>

                        <div className="px-6 py-4 border-t border-[var(--border-color)] bg-[var(--bg-primary)]/50 flex justify-end gap-3">
                            <button
                                onClick={handleCloseModal}
                                className="px-4 py-2 text-sm font-medium text-[var(--text-secondary)] hover:text-white transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleSave}
                                className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg transition-colors shadow-lg shadow-indigo-500/20"
                            >
                                Save Changes
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}
