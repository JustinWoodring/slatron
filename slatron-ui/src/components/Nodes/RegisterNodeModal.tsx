import React, { useState } from 'react'
import { apiClient } from '../../api/client'

interface RegisterNodeModalProps {
    isOpen: boolean
    onClose: () => void
    onSuccess: () => void
}

export const RegisterNodeModal = ({ isOpen, onClose, onSuccess }: RegisterNodeModalProps) => {
    const [name, setName] = useState('')
    const [result, setResult] = useState<{ node: any, secret_key: string } | null>(null)
    const [error, setError] = useState<string | null>(null)

    if (!isOpen) return null

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        setError(null)
        try {
            const response = await apiClient.post('/api/nodes', { name })
            setResult(response.data)
            onSuccess() // Refresh list
        } catch (e) {
            console.error(e)
            setError('Failed to register node')
        }
    }

    const handleClose = () => {
        setName('')
        setResult(null)
        setError(null)
        onClose()
    }

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border-color)] w-full max-w-md overflow-hidden animate-fade-in shadow-2xl">
                <div className="p-6 border-b border-[var(--border-color)] flex justify-between items-center bg-[var(--bg-tertiary)]">
                    <h2 className="text-xl font-bold text-white">Register Node</h2>
                    <button onClick={handleClose} className="text-[var(--text-secondary)] hover:text-white transition-colors">
                        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div className="p-6">
                    {result ? (
                        <div className="space-y-4">
                            <div className="p-4 bg-emerald-500/10 border border-emerald-500/20 rounded-lg text-emerald-400 text-sm">
                                Node registered successfully!
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-2">Secret Key</label>
                                <div className="p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg font-mono text-sm break-all text-white select-all">
                                    {result.secret_key}
                                </div>
                                <p className="text-xs text-yellow-500 mt-2">
                                    ⚠️ Save this key now! It will not be shown again.
                                </p>
                            </div>
                            <button
                                onClick={handleClose}
                                className="w-full btn-primary"
                            >
                                Done
                            </button>
                        </div>
                    ) : (
                        <form onSubmit={handleSubmit} className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Node Name</label>
                                <input
                                    type="text"
                                    required
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white focus:border-indigo-500 outline-none"
                                    value={name}
                                    onChange={e => setName(e.target.value)}
                                    placeholder="e.g. Living Room TV"
                                />
                            </div>

                            {error && (
                                <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-sm">
                                    {error}
                                </div>
                            )}

                            <div className="flex justify-end gap-3 pt-2">
                                <button
                                    type="button"
                                    onClick={handleClose}
                                    className="px-4 py-2 rounded-lg text-sm font-medium text-[var(--text-secondary)] hover:text-white hover:bg-[var(--bg-primary)] transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    className="btn-primary"
                                >
                                    Register
                                </button>
                            </div>
                        </form>
                    )}
                </div>
            </div>
        </div>
    )
}
