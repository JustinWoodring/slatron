import React, { useState, useEffect } from 'react'
import { useContentStore, ContentItem } from '../../stores/contentStore'
import { useScriptStore } from '../../stores/scriptStore'

interface CreateContentModalProps {
    isOpen: boolean
    onClose: () => void
    editingContent?: ContentItem
}

export default function CreateContentModal({ isOpen, onClose, editingContent }: CreateContentModalProps) {
    const { createContent, updateContent } = useContentStore()
    const { scripts, fetchScripts, executeScript } = useScriptStore()

    // UI State
    const [mode, setMode] = useState<'manual' | 'loader'>('manual')
    const [isLoading, setIsLoading] = useState(false)
    const [loaderError, setLoaderError] = useState<string | null>(null)

    // Loader State
    const [selectedScriptId, setSelectedScriptId] = useState<number | undefined>()
    const [scriptParams, setScriptParams] = useState('{}')

    // Content Form State
    const [formData, setFormData] = useState({
        title: '',
        description: '',
        content_type: 'local_file',
        content_path: '',
        duration_minutes: 0,
        tags: '',
        node_accessibility: 'public',
        adapter_id: undefined as number | undefined,
        transformer_scripts: [] as { id: number, args: Record<string, any> }[]
    })

    useEffect(() => {
        if (isOpen) {
            fetchScripts()
            if (editingContent) {
                let transformers: any[] = []
                try {
                    if (editingContent.transformer_scripts) {
                        transformers = JSON.parse(editingContent.transformer_scripts)
                    }
                } catch (e) {
                    console.warn("Failed to parse transformer_scripts", e)
                }

                setFormData({
                    title: editingContent.title,
                    description: editingContent.description || '',
                    content_type: editingContent.content_type,
                    content_path: editingContent.content_path,
                    duration_minutes: editingContent.duration_minutes || 0,
                    tags: editingContent.tags || '',
                    node_accessibility: editingContent.node_accessibility || 'public',
                    adapter_id: undefined, // TODO: Handle adapter_id if present in ContentItem
                    transformer_scripts: transformers.map(t => {
                        if (typeof t === 'number') return { id: t, args: {} }
                        return t as { id: number, args: Record<string, any> }
                    })
                })
            } else {
                // Reset for create
                setFormData({
                    title: '',
                    description: '',
                    content_type: 'local_file',
                    content_path: '',
                    duration_minutes: 0,
                    tags: '',
                    node_accessibility: 'public',
                    adapter_id: undefined,
                    transformer_scripts: []
                })
            }
        }
    }, [isOpen, editingContent])

    const loaders = scripts.filter(s => s.script_type === 'content_loader')

    const resetForm = () => {
        setFormData({
            title: '',
            description: '',
            content_type: 'local_file',
            content_path: '',
            duration_minutes: 0,
            tags: '',
            node_accessibility: 'public',
            adapter_id: undefined,
            transformer_scripts: []
        })
        setMode('manual')
        setSelectedScriptId(undefined)
        setScriptParams('{}')
        setLoaderError(null)
    }

    const handleClose = () => {
        resetForm()
        onClose()
    }

    const handleRunLoader = async () => {
        if (!selectedScriptId) return
        setIsLoading(true)
        setLoaderError(null)

        try {
            let params = {}
            try {
                params = JSON.parse(scriptParams)
            } catch (e) {
                throw new Error("Invalid JSON parameters")
            }

            const res = await executeScript(selectedScriptId, params)

            if (res.success && res.result) {
                try {
                    const data = JSON.parse(res.result)

                    setFormData(prev => ({
                        ...prev,
                        title: data.title || prev.title,
                        description: data.description || prev.description,
                        content_path: data.path || data.url || prev.content_path,
                        duration_minutes: data.duration || prev.duration_minutes,
                        content_type: data.type || (data.path ? 'local_file' : 'remote_url')
                    }))
                    setMode('manual')
                } catch (e) {
                    setLoaderError("Failed to parse script output. Ensure script returns a JSON string.")
                }
            } else {
                setLoaderError(res.error || "Execution failed")
            }
        } catch (e) {
            setLoaderError(String(e))
        } finally {
            setIsLoading(false)
        }
    }

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        try {
            const data = {
                ...formData,
                duration_minutes: formData.duration_minutes || null,
                description: formData.description || null,
                tags: formData.tags || null,
                node_accessibility: formData.node_accessibility || null,
                transformer_scripts: formData.transformer_scripts.length > 0
                    ? JSON.stringify(formData.transformer_scripts)
                    : null
            }

            if (editingContent) {
                await updateContent(editingContent.id, data)
            } else {
                await createContent(data)
            }
            handleClose()
        } catch (error) {
            console.error('Failed to save content:', error)
        }
    }

    if (!isOpen) return null

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl w-full max-w-lg shadow-2xl animate-fade-in flex flex-col max-h-[90vh]">

                {/* Header with Tabs */}
                <div className="flex border-b border-[var(--border-color)]">
                    <button
                        onClick={() => setMode('manual')}
                        className={`flex-1 py-4 text-sm font-medium transition-colors ${mode === 'manual' ? 'text-emerald-400 border-b-2 border-emerald-400 bg-[var(--bg-tertiary)]' : 'text-[var(--text-secondary)] hover:text-white'}`}
                    >
                        Manual Entry
                    </button>
                    <button
                        onClick={() => setMode('loader')}
                        className={`flex-1 py-4 text-sm font-medium transition-colors ${mode === 'loader' ? 'text-emerald-400 border-b-2 border-emerald-400 bg-[var(--bg-tertiary)]' : 'text-[var(--text-secondary)] hover:text-white'}`}
                    >
                        Import via Script
                    </button>
                </div>

                <div className="p-6 overflow-y-auto custom-scrollbar">
                    {mode === 'loader' ? (
                        <div className="space-y-4">
                            <p className="text-sm text-[var(--text-secondary)]">
                                Run a content loader script to automatically fetch metadata and fill the form.
                            </p>

                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Select Loader</label>
                                <select
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                    value={selectedScriptId || ''}
                                    onChange={e => setSelectedScriptId(Number(e.target.value))}
                                >
                                    <option value="">Select a script...</option>
                                    {loaders.map(s => (
                                        <option key={s.id} value={s.id}>{s.name}</option>
                                    ))}
                                </select>
                            </div>

                            {selectedScriptId && (() => {
                                const selectedScript = loaders.find(s => s.id === selectedScriptId)
                                let schemaFields: any[] = []
                                try {
                                    if (selectedScript?.parameters_schema) {
                                        const schema = JSON.parse(selectedScript.parameters_schema)
                                        // Simple Key-Value schema support for now
                                        // { "url": "string", "quality": "string" }
                                        // Or { "properties": { ... } }
                                        const props = schema.properties || schema
                                        schemaFields = Object.keys(props).map(key => ({
                                            name: key,
                                            label: key.charAt(0).toUpperCase() + key.slice(1),
                                            type: 'text' // Auto-detect based on schema type if possible
                                        }))
                                    }
                                } catch (e) {
                                    console.warn("Invalid schema", e)
                                }

                                return schemaFields.length > 0 ? (
                                    <div className="space-y-4 p-4 rounded-lg bg-[var(--bg-primary)]/50 border border-[var(--border-color)]">
                                        <p className="text-xs text-[var(--text-secondary)] uppercase font-bold mb-2">Script Parameters</p>
                                        {schemaFields.map(field => (
                                            <div key={field.name}>
                                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">{field.label}</label>
                                                <input
                                                    type="text"
                                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                                    value={(() => {
                                                        try {
                                                            const p = JSON.parse(scriptParams)
                                                            return p[field.name] || ''
                                                        } catch { return '' }
                                                    })()}
                                                    onChange={e => {
                                                        try {
                                                            const p = JSON.parse(scriptParams)
                                                            p[field.name] = e.target.value
                                                            setScriptParams(JSON.stringify(p))
                                                        } catch {
                                                            const p = { [field.name]: e.target.value }
                                                            setScriptParams(JSON.stringify(p))
                                                        }
                                                    }}
                                                />
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <div>
                                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Parameters (JSON)</label>
                                        <textarea
                                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white bg-[#1e1e1e] font-mono text-xs focus:border-indigo-500 outline-none resize-y h-32"
                                            value={scriptParams}
                                            onChange={e => setScriptParams(e.target.value)}
                                            placeholder="{}"
                                        />
                                    </div>
                                )
                            })()}

                            {loaderError && (
                                <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-xs">
                                    {loaderError}
                                </div>
                            )}

                            <div className="flex justify-end pt-2">
                                <button
                                    type="button"
                                    onClick={handleRunLoader}
                                    disabled={!selectedScriptId || isLoading}
                                    className="btn-primary w-full flex justify-center items-center gap-2"
                                >
                                    {isLoading && (
                                        <svg className="animate-spin h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                        </svg>
                                    )}
                                    Run Loader
                                </button>
                            </div>
                        </div>
                    ) : (
                        <form id="content-form" onSubmit={handleSubmit} className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Title</label>
                                <input
                                    type="text"
                                    required
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                    value={formData.title}
                                    onChange={e => setFormData({ ...formData, title: e.target.value })}
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Description</label>
                                <textarea
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none h-20"
                                    value={formData.description}
                                    onChange={e => setFormData({ ...formData, description: e.target.value })}
                                />
                            </div>

                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Type</label>
                                    <select
                                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                        value={formData.content_type}
                                        onChange={e => setFormData({ ...formData, content_type: e.target.value })}
                                    >
                                        <option value="local_file">Local File</option>
                                        <option value="remote_url">Remote URL</option>
                                    </select>
                                </div>
                                <div>
                                    <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Duration (min)</label>
                                    <input
                                        type="number"
                                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                        value={formData.duration_minutes}
                                        onChange={e => setFormData({ ...formData, duration_minutes: parseInt(e.target.value) || 0 })}
                                    />
                                </div>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Path / URL</label>
                                <input
                                    type="text"
                                    required
                                    placeholder="/path/to/file.mp4 or https://..."
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none font-mono"
                                    value={formData.content_path}
                                    onChange={e => setFormData({ ...formData, content_path: e.target.value })}
                                />
                            </div>

                            {/* Transformers Selection */}
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-2">Transformers</label>
                                <div className="space-y-2 p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg max-h-32 overflow-y-auto custom-scrollbar">
                                    <div className="space-y-4">
                                        {/* Add Transformer */}
                                        <div className="flex gap-2">
                                            <select
                                                className="flex-1 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                                onChange={(e) => {
                                                    const scriptId = Number(e.target.value);
                                                    if (scriptId) {
                                                        const exists = formData.transformer_scripts.some(t => t.id === scriptId);
                                                        if (!exists) {
                                                            setFormData({
                                                                ...formData,
                                                                transformer_scripts: [...formData.transformer_scripts, { id: scriptId, args: {} }]
                                                            });
                                                        }
                                                        e.target.value = "";
                                                    }
                                                }}
                                            >
                                                <option value="">Add a transformer...</option>
                                                {scripts.filter(s => s.script_type === 'transformer').map(s => (
                                                    <option key={s.id} value={s.id} disabled={formData.transformer_scripts.some(t => t.id === s.id)}>
                                                        {s.name}
                                                    </option>
                                                ))}
                                            </select>
                                        </div>

                                        {/* List Configured Transformers */}
                                        <div className="space-y-3">
                                            {formData.transformer_scripts.map((entry, index) => {
                                                const script = scripts.find(s => s.id === entry.id);
                                                return (
                                                    <div key={entry.id} className="p-3 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-lg">
                                                        <div className="flex justify-between items-center mb-2">
                                                            <span className="text-sm font-medium text-white">{script?.name || `Script #${entry.id}`}</span>
                                                            <button
                                                                type="button"
                                                                onClick={() => {
                                                                    const newScripts = [...formData.transformer_scripts];
                                                                    newScripts.splice(index, 1);
                                                                    setFormData({ ...formData, transformer_scripts: newScripts });
                                                                }}
                                                                className="text-red-400 hover:text-red-300 text-xs"
                                                            >
                                                                Remove
                                                            </button>
                                                        </div>

                                                        {/* Simple Args Editor (JSON) */}
                                                        <div>
                                                            <label className="text-xs text-[var(--text-secondary)] block mb-1">Arguments (JSON)</label>
                                                            <textarea
                                                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-xs text-white font-mono h-16 resize-none focus:border-indigo-500 outline-none"
                                                                value={JSON.stringify(entry.args)}
                                                                onChange={(e) => {
                                                                    try {
                                                                        const args = JSON.parse(e.target.value);
                                                                        const newScripts = [...formData.transformer_scripts];
                                                                        newScripts[index] = { ...entry, args };
                                                                        setFormData({ ...formData, transformer_scripts: newScripts });
                                                                    } catch {
                                                                        // Allow typing, but maybe store local string state if validating?
                                                                        // For simplicity in MVP, we just try to parse.
                                                                    }
                                                                }}
                                                            />
                                                            {/* Helper to parse text area properly: we need local state for text area or accept invalid JSON temporarily?
                                                                React controlled input with JSON.stringify(obj) is hard because typing '"' breaks parse.
                                                                Let's use a simpler approach: Just one text input for "params" string?
                                                                Or Key-Value fields.
                                                                Let's do dynamic Key-Value pairs.
                                                            */}
                                                        </div>
                                                        <div className="mt-2 text-xs text-[var(--text-secondary)]">
                                                            Edit args directly above. (Note: Valid JSON required for now. Future: Key/Value editor)
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div className="flex justify-end gap-3 pt-4">
                                <button
                                    type="button"
                                    onClick={handleClose}
                                    className="px-4 py-2 text-sm text-[var(--text-secondary)] hover:text-white transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    className="btn-primary"
                                >
                                    Save Content
                                </button>
                            </div>
                        </form>
                    )}
                </div>
            </div>
        </div>
    )
}
