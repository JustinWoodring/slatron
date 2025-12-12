import { useState, useEffect } from 'react'
import { apiClient } from '../../api/client'

interface Node {
    id: number
    name: string
    status: string
}

interface NodeSettingsModalProps {
    isOpen: boolean
    onClose: () => void
    node: Node | null
    onSuccess: () => void
}

export function NodeSettingsModal({ isOpen, onClose, node, onSuccess }: NodeSettingsModalProps) {
    const [name, setName] = useState('')
    const [loading, setLoading] = useState(false)

    useEffect(() => {
        if (node) {
            setName(node.name)
        }
    }, [node])

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!node) return

        setLoading(true)
        try {
            await apiClient.put(`/api/nodes/${node.id}`, { name })
            onSuccess()
            onClose()
        } catch (error) {
            console.error('Failed to update node:', error)
            alert('Failed to update node')
        } finally {
            setLoading(false)
        }
    }

    const handleDelete = async () => {
        if (!node || !confirm('Are you sure you want to delete this node? This action cannot be undone.')) return

        setLoading(true)
        try {
            await apiClient.delete(`/api/nodes/${node.id}`)
            onSuccess()
            onClose()
        } catch (error) {
            console.error('Failed to delete node:', error)
            alert('Failed to delete node')
        } finally {
            setLoading(false)
        }
    }

    if (!isOpen || !node) return null

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
            <div className="glass-panel p-6 rounded-xl w-full max-w-md border border-[var(--border-color)]">
                <h2 className="text-xl font-bold text-white mb-4">Node Settings</h2>

                <form onSubmit={handleSubmit} className="space-y-6">
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                            Node Name
                        </label>
                        <input
                            type="text"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            className="w-full bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500"
                            placeholder="Enter node name"
                            required
                        />
                    </div>

                    <div className="flex gap-3 pt-2">
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex-1 px-4 py-2 rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:bg-[var(--bg-secondary)] transition-colors"
                            disabled={loading}
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            className="flex-1 btn-primary"
                            disabled={loading}
                        >
                            {loading ? 'Saving...' : 'Save Changes'}
                        </button>
                    </div>
                </form>

                <div className="mt-8 pt-6 border-t border-[var(--border-color)]">
                    <h3 className="text-sm font-bold text-red-400 mb-2">Danger Zone</h3>
                    <p className="text-xs text-[var(--text-secondary)] mb-4">
                        Deleting a node will remove it from the system. Previous logs may be retained but schedule assignments will be lost.
                    </p>
                    <button
                        type="button"
                        onClick={handleDelete}
                        className="w-full px-4 py-2 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 hover:bg-red-500/20 transition-colors text-sm font-medium"
                        disabled={loading}
                    >
                        Delete Node
                    </button>
                </div>
            </div>
        </div>
    )
}
