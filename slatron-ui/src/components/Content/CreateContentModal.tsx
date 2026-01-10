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
    const [mode, setMode] = useState<'manual' | 'loader' | 'bulk_review' | 'import_report'>('manual')
    const [isLoading, setIsLoading] = useState(false)
    const [loaderError, setLoaderError] = useState<string | null>(null)
    const [foundItems, setFoundItems] = useState<any[]>([])
    const [importProgress, setImportProgress] = useState<{ current: number, total: number } | null>(null)
    const [importResults, setImportResults] = useState<{ success: number, failed: number, errors: string[] } | null>(null)

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
        is_dj_accessible: false,
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
                    is_dj_accessible: editingContent.is_dj_accessible || false,
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
                    is_dj_accessible: false,
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
            is_dj_accessible: false,
            adapter_id: undefined,
            transformer_scripts: []
        })
        setMode('manual')
        setSelectedScriptId(undefined)
        setScriptParams('{}')
        setLoaderError(null)
        setFoundItems([])
        setImportProgress(null)
        setImportResults(null)
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

                    if (Array.isArray(data)) {
                        // Bulk Import Mode
                        if (data.length === 0) {
                            setLoaderError("Script returned an empty list.")
                            return
                        }
                        setFoundItems(data)
                        setMode('bulk_review')
                    } else {
                        // Single Item Mode
                        setFormData(prev => ({
                            ...prev,
                            title: data.title || prev.title,
                            description: data.description || prev.description,
                            content_path: data.path || data.url || prev.content_path,
                            duration_minutes: data.duration_minutes || data.duration || prev.duration_minutes, // Fix: support duration_minutes key
                            content_type: data.type || (data.path || data.url ? 'remote_url' : 'local_file') // Improve type detection
                        }))
                        setMode('manual')
                    }
                } catch (e) {
                    setLoaderError("Failed to parse script output. Ensure script returns a valid JSON string.")
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

    const handleBulkImport = async () => {
        if (foundItems.length === 0) return
        setIsLoading(true)
        setImportProgress({ current: 0, total: foundItems.length })

        let successCount = 0
        let failures: string[] = []

        for (let i = 0; i < foundItems.length; i++) {
            const item = foundItems[i]
            try {
                // Map script item to content data
                const data = {
                    title: item.title || "Untitled",
                    description: item.description || null,
                    content_type: item.content_type || item.type || (item.content_path || item.url ? 'remote_url' : 'local_file'),
                    content_path: item.content_path || item.url || item.path || "",
                    duration_minutes: Math.round(item.duration_minutes || item.duration || 0),
                    tags: item.tags || null,
                    node_accessibility: 'public',
                    is_dj_accessible: false, // Default for bulk import
                    transformer_scripts: null,
                    adapter_id: undefined
                }

                if (!data.content_path) {
                    failures.push(`Item "${data.title}": Missing content path`)
                    continue;
                }

                await createContent(data as any)
                successCount++
            } catch (e: any) {
                console.error(`Failed to import item ${i}`, e)
                const msg = e.response?.data?.error || e.message || "Unknown error"
                failures.push(`Item "${item.title || i}": ${msg}`)
            }
            setImportProgress({ current: i + 1, total: foundItems.length })
        }

        setIsLoading(false)
        setImportProgress(null)

        setImportResults({
            success: successCount,
            failed: failures.length,
            errors: failures
        })
        setMode('import_report')
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
                is_dj_accessible: formData.is_dj_accessible,
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
                        className={`flex-1 py-4 text-sm font-medium transition-colors ${mode === 'loader' || mode === 'bulk_review' ? 'text-emerald-400 border-b-2 border-emerald-400 bg-[var(--bg-tertiary)]' : 'text-[var(--text-secondary)] hover:text-white'}`}
                    >
                        Import via Script
                    </button>
                </div>

                <div className="p-6 overflow-y-auto custom-scrollbar">
                    {mode === 'bulk_review' ? (
                        <div className="space-y-4">
                            <div className="flex justify-between items-center">
                                <h3 className="text-white font-medium">Review Import ({foundItems.length} items)</h3>
                                <button onClick={() => setMode('loader')} className="text-xs text-[var(--text-secondary)] hover:text-white">
                                    Back to Loader
                                </button>
                            </div>

                            <div className="bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg overflow-hidden max-h-64 overflow-y-auto custom-scrollbar">
                                <table className="w-full text-left text-sm text-[var(--text-secondary)]">
                                    <thead className="bg-[#1e1e1e] sticky top-0">
                                        <tr>
                                            <th className="p-2 font-medium text-xs uppercase">Title</th>
                                            <th className="p-2 font-medium text-xs uppercase text-right">Dur (m)</th>
                                        </tr>
                                    </thead>
                                    <tbody className="divide-y divide-[var(--border-color)]">
                                        {foundItems.map((item, idx) => (
                                            <tr key={idx} className="hover:bg-white/5">
                                                <td className="p-2 truncate max-w-[200px]" title={item.title}>{item.title}</td>
                                                <td className="p-2 text-right font-mono text-xs">
                                                    {(item.duration_minutes || item.duration || 0).toFixed(1)}
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>

                            {importProgress && (
                                <div className="space-y-1">
                                    <div className="flex justify-between text-xs text-[var(--text-secondary)]">
                                        <span>Importing...</span>
                                        <span>{importProgress.current} / {importProgress.total}</span>
                                    </div>
                                    <div className="w-full bg-[var(--bg-primary)] rounded-full h-1.5">
                                        <div
                                            className="bg-emerald-500 h-1.5 rounded-full transition-all duration-300"
                                            style={{ width: `${(importProgress.current / importProgress.total) * 100}%` }}
                                        />
                                    </div>
                                </div>
                            )}

                            <div className="flex justify-end gap-3 pt-2">
                                <button
                                    type="button"
                                    onClick={handleClose}
                                    disabled={isLoading}
                                    className="px-4 py-2 text-sm text-[var(--text-secondary)] hover:text-white transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="button"
                                    onClick={handleBulkImport}
                                    disabled={isLoading}
                                    className="btn-primary flex items-center gap-2"
                                >
                                    {isLoading && <span className="animate-spin text-white">⟳</span>}
                                    Import All
                                </button>
                            </div>
                        </div>
                    ) : mode === 'loader' ? (
                        <div className="space-y-4">
                            <p className="text-sm text-[var(--text-secondary)]">
                                Run a content loader script. If it returns a list, you can bulk import.
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
                    ) : mode === 'import_report' && importResults ? (
                        <div className="space-y-4">
                            <div className="text-center py-4">
                                {importResults.failed === 0 ? (
                                    <div className="text-emerald-400 mb-2">
                                        <svg className="w-12 h-12 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                        </svg>
                                        <h3 className="text-lg font-medium mt-2">Import Successful</h3>
                                    </div>
                                ) : (
                                    <div className="text-amber-400 mb-2">
                                        <svg className="w-12 h-12 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                        </svg>
                                        <h3 className="text-lg font-medium mt-2">Import Completed with Errors</h3>
                                    </div>
                                )}
                                <p className="text-[var(--text-secondary)]">
                                    Successfully imported <strong>{importResults.success}</strong> items.
                                    <br />
                                    Failed to import <strong>{importResults.failed}</strong> items.
                                </p>
                            </div>

                            {importResults.failed > 0 && (
                                <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
                                    <h4 className="text-red-400 text-sm font-medium mb-2">Failures</h4>
                                    <div className="max-h-40 overflow-y-auto custom-scrollbar space-y-1">
                                        {importResults.errors.map((err, idx) => (
                                            <div key={idx} className="text-xs text-red-300 font-mono border-b border-red-500/10 pb-1 mb-1 last:border-0">
                                                • {err}
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}

                            <div className="flex justify-center pt-2">
                                <button
                                    type="button"
                                    onClick={handleClose}
                                    className="btn-primary w-full"
                                >
                                    Close
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

                            <div className="flex items-center">
                                <label className="flex items-center cursor-pointer">
                                    <input
                                        type="checkbox"
                                        className="form-checkbox h-4 w-4 text-indigo-500 rounded border-[var(--border-color)] bg-[var(--bg-primary)] focus:ring-indigo-500"
                                        checked={formData.is_dj_accessible}
                                        onChange={e => setFormData({ ...formData, is_dj_accessible: e.target.checked })}
                                    />
                                    <span className="ml-2 text-sm text-white">DJ Accessible</span>
                                </label>
                                <span className="ml-2 text-xs text-[var(--text-secondary)]">(Allow AI DJs to pick this track)</span>
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
