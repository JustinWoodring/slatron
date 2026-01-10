import { useEffect, useState } from 'react'
import Editor from '@monaco-editor/react'
import { useParams, useNavigate } from 'react-router-dom'
import { useScriptStore } from '../stores/scriptStore'
import { useAuthStore } from '../stores/authStore'

import { apiClient } from '../api/client'

export default function ScriptEditorPage() {
    const { user } = useAuthStore()
    const isEditor = user?.role === 'admin' || user?.role === 'editor'
    const { id } = useParams<{ id: string }>()
    const navigate = useNavigate()
    const { scripts, fetchScripts, updateScript, deleteScript, executeScript } = useScriptStore()

    const [script, setScript] = useState<any>(null)
    const [content, setContent] = useState('')
    const [isDirty, setIsDirty] = useState(false)
    const [isGlobal, setIsGlobal] = useState(false)
    const [allGlobalScripts, setAllGlobalScripts] = useState<string[]>([])

    const [activeTab, setActiveTab] = useState<'source' | 'schema'>('source')
    const [schemaContent, setSchemaContent] = useState('{}')
    // Test execution state
    const [testParams, setTestParams] = useState('{}')
    const [testResult, setTestResult] = useState<string | null>(null)
    const [testCommands, setTestCommands] = useState<string[]>([])
    const [testError, setTestError] = useState<string | null>(null)
    const [isExecuting, setIsExecuting] = useState(false)

    useEffect(() => {
        if (!scripts.length) {
            fetchScripts()
        }
        fetchGlobals()
    }, [])

    const fetchGlobals = async () => {
        try {
            const res = await apiClient.get<any[]>('/api/settings')
            const setting = res.data.find((s: any) => s.key === 'global_active_scripts')
            if (setting) {
                try {
                    const parsed = JSON.parse(setting.value)
                    setAllGlobalScripts(parsed)
                } catch (e) {
                    console.error("Failed to parse global scripts setting", e)
                }
            }
        } catch (e) {
            console.error("Failed to fetch settings", e)
        }
    }

    useEffect(() => {
        if (id && scripts.length) {
            const found = scripts.find(s => s.id === parseInt(id))
            if (found) {
                setScript(found)
                setContent(found.script_content)
                setSchemaContent(found.parameters_schema || '{}')
            }
        }
    }, [id, scripts])

    useEffect(() => {
        if (script && allGlobalScripts.includes(script.name)) {
            setIsGlobal(true)
        } else {
            setIsGlobal(false)
        }
    }, [script, allGlobalScripts])

    const handleToggleGlobal = async () => {
        if (!script) return
        const newStatus = !isGlobal
        setIsGlobal(newStatus)

        let newList = [...allGlobalScripts]
        if (newStatus) {
            if (!newList.includes(script.name)) newList.push(script.name)
        } else {
            newList = newList.filter(n => n !== script.name)
        }
        setAllGlobalScripts(newList)

        try {
            await apiClient.put('/api/settings/global_active_scripts', {
                key: 'global_active_scripts',
                value: JSON.stringify(newList),
                description: 'JSON array of Script Names to execute for every content item.'
            })
        } catch (e) {
            console.error("Failed to update global scripts", e)
            alert("Failed to update global status")
            // Revert
            setIsGlobal(!newStatus)
            fetchGlobals()
        }
    }

    const handleSave = async () => {
        if (!script) return
        try {
            // Validate JSON schema if present
            try {
                if (schemaContent.trim()) {
                    JSON.parse(schemaContent)
                }
            } catch (e) {
                alert('Invalid JSON in Parameters Schema')
                return
            }

            await updateScript(script.id, {
                ...script,
                script_content: content,
                parameters_schema: schemaContent
            })
            setIsDirty(false)
        } catch (e) {
            console.error(e)
            alert('Failed to save script')
        }
    }

    const handleDelete = async () => {
        if (!script || !window.confirm('Are you sure you want to delete this script?')) return
        try {
            await deleteScript(script.id)
            navigate('/scripts')
        } catch (e) {
            console.error(e)
            alert('Failed to delete script')
        }
    }

    const handleTestRun = async () => {
        if (!script) return
        setIsExecuting(true)
        setTestResult(null)
        setTestCommands([])
        setTestError(null)
        try {
            let params = {}
            try {
                params = JSON.parse(testParams)
            } catch (e) {
                setTestError('Invalid JSON params')
                setIsExecuting(false)
                return
            }

            if (isDirty) {
                if (window.confirm('Script has unsaved changes. Save and run?')) {
                    await handleSave()
                } else {
                    setIsExecuting(false)
                    return
                }
            }

            const res = await executeScript(script.id, params)
            if (res.success) {
                setTestResult(res.result || 'Success (No output)')
                setTestCommands(res.mpv_commands || [])
            } else {
                setTestError(res.error || 'Unknown error')
            }
        } catch (e) {
            setTestError(String(e))
        } finally {
            setIsExecuting(false)
        }
    }

    if (!script) {
        return <div className="p-6 text-center text-gray-500">Loading script...</div>
    }

    return (
        <div className="h-full flex flex-col p-6 gap-6">
            {/* Header */}
            <div className="flex justify-between items-center">
                <div className="flex items-center gap-4">
                    <button onClick={() => navigate('/scripts')} className="text-[var(--text-secondary)] hover:text-white">
                        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                        </svg>
                    </button>
                    <div>
                        <h1 className="text-2xl font-bold bg-gradient-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">
                            {script.name}
                        </h1>
                        <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
                            <span>{script.script_type}</span>
                            {isGlobal && <span className="text-amber-400 font-bold px-2 py-0.5 bg-amber-400/10 rounded-full text-xs">GLOBAL</span>}
                        </div>
                    </div>
                </div>
                <div className="flex items-center gap-3">
                    {isEditor && (
                        <button
                            onClick={handleToggleGlobal}
                            className={`px-4 py-2 rounded-lg transition-colors border ${isGlobal ? 'bg-amber-500/10 text-amber-400 border-amber-500/50 hover:bg-amber-500/20' : 'text-[var(--text-secondary)] border-[var(--border-color)] hover:text-white hover:border-white/50'}`}
                            title="Run this script on ALL content items automatically"
                        >
                            {isGlobal ? 'Global: ON' : 'Global: OFF'}
                        </button>
                    )}

                    {isEditor && (
                        <button
                            onClick={handleDelete}
                            className="px-4 py-2 rounded-lg text-red-400 hover:bg-red-500/10 transition-colors"
                        >
                            Delete
                        </button>
                    )}

                    {isEditor && (
                        <button
                            onClick={handleSave}
                            disabled={!isDirty}
                            className={`btn-primary ${!isDirty ? 'opacity-50 cursor-not-allowed' : ''}`}
                        >
                            {isDirty ? 'Save Changes' : 'Saved'}
                        </button>
                    )}
                </div>
            </div>

            <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 min-h-0">
                {/* Editor Column */}
                <div className="lg:col-span-2 flex flex-col gap-2">
                    {/* Tabs */}
                    <div className="flex gap-2">
                        <button
                            onClick={() => setActiveTab('source')}
                            className={`px-4 py-2 rounded-t-lg text-sm font-medium transition-colors ${activeTab === 'source' ? 'bg-[var(--bg-secondary)] text-white border-t border-x border-[var(--border-color)]' : 'text-[var(--text-secondary)] hover:text-white'}`}
                        >
                            Script Source
                        </button>
                        <button
                            onClick={() => setActiveTab('schema')}
                            className={`px-4 py-2 rounded-t-lg text-sm font-medium transition-colors ${activeTab === 'schema' ? 'bg-[var(--bg-secondary)] text-white border-t border-x border-[var(--border-color)]' : 'text-[var(--text-secondary)] hover:text-white'}`}
                        >
                            Parameters Schema
                        </button>
                    </div>

                    <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-b-xl rounded-tr-xl flex-1 flex flex-col overflow-hidden relative min-h-[500px]">
                        {activeTab === 'source' ? (
                            <>
                                <div className="p-2 border-b border-[var(--border-color)] bg-[var(--bg-tertiary)] flex justify-between items-center z-10">
                                    <span className="text-xs font-mono text-[var(--text-secondary)]">main.rhai</span>
                                    <span className="text-xs text-[var(--text-secondary)]">Rhai Script</span>
                                </div>
                                <div className="flex-1 relative">
                                    <Editor
                                        height="100%"
                                        defaultLanguage="rust"
                                        theme="vs-dark"
                                        value={content}
                                        onChange={(value: string | undefined) => {
                                            setContent(value || '')
                                            setIsDirty(true)
                                        }}
                                        options={{
                                            minimap: { enabled: false },
                                            fontSize: 14,
                                            padding: { top: 16 },
                                            fontFamily: 'JetBrains Mono, monospace',
                                            scrollBeyondLastLine: false,
                                            automaticLayout: true,
                                            tabSize: 4,
                                            readOnly: !isEditor
                                        }}
                                    />
                                </div>
                            </>
                        ) : (
                            <>
                                <div className="p-2 border-b border-[var(--border-color)] bg-[var(--bg-tertiary)] flex justify-between items-center z-10">
                                    <span className="text-xs font-mono text-[var(--text-secondary)]">schema.json</span>
                                    <span className="text-xs text-[var(--text-secondary)]">JSON Schema</span>
                                </div>
                                <div className="flex-1 relative">
                                    <Editor
                                        height="100%"
                                        defaultLanguage="json"
                                        theme="vs-dark"
                                        value={schemaContent}
                                        onChange={(value: string | undefined) => {
                                            setSchemaContent(value || '{}')
                                            setIsDirty(true)
                                        }}
                                        options={{
                                            minimap: { enabled: false },
                                            fontSize: 14,
                                            padding: { top: 16 },
                                            fontFamily: 'JetBrains Mono, monospace',
                                            scrollBeyondLastLine: false,
                                            automaticLayout: true,
                                            tabSize: 2,
                                            readOnly: !isEditor
                                        }}
                                    />
                                </div>
                            </>
                        )}
                    </div>
                </div>

                {/* Execution/Params Column */}
                <div className="flex flex-col gap-4">
                    <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl p-4 flex flex-col gap-4">
                        <h3 className="font-bold text-white flex items-center gap-2">
                            <svg className="w-4 h-4 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            Test Execution
                        </h3>

                        <div>
                            <label className="text-xs font-medium text-[var(--text-secondary)] mb-1 block">Parameters (JSON)</label>
                            <textarea
                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 font-mono text-xs h-32 text-gray-300 focus:border-indigo-500 outline-none resize-y"
                                value={testParams}
                                onChange={e => setTestParams(e.target.value)}
                                disabled={!isEditor}
                            />
                        </div>

                        {isEditor && (
                            <button
                                onClick={handleTestRun}
                                disabled={isExecuting}
                                className="w-full bg-emerald-600 hover:bg-emerald-700 text-white py-2 rounded-lg font-medium transition-colors disabled:opacity-50"
                            >
                                {isExecuting ? 'Running...' : 'Run Script'}
                            </button>
                        )}
                    </div>

                    {/* Output */}
                    {(testResult || testError || testCommands.length > 0) && (
                        <div className={`bg-[var(--bg-secondary)] border ${testError ? 'border-red-500/50' : 'border-emerald-500/50'} rounded-xl p-4 flex flex-col gap-2 overflow-hidden`}>
                            <h3 className={`font-bold text-sm ${testError ? 'text-red-400' : 'text-emerald-400'}`}>
                                {testError ? 'Execution Failed' : 'Output'}
                            </h3>
                            <div className="overflow-auto max-h-60 custom-scrollbar flex flex-col gap-2">
                                {testError && (
                                    <pre className="text-xs font-mono whitespace-pre-wrap text-red-300">
                                        {testError}
                                    </pre>
                                )}
                                {testResult && (
                                    <pre className="text-xs font-mono whitespace-pre-wrap text-gray-300">
                                        {testResult}
                                    </pre>
                                )}
                                {testCommands.length > 0 && (
                                    <div className="border-t border-gray-700/50 pt-2 mt-1">
                                        <p className="text-[10px] text-gray-500 uppercase font-bold mb-1">MPV Commands</p>
                                        <div className="space-y-1">
                                            {testCommands.map((cmd, i) => (
                                                <div key={i} className="text-xs font-mono text-cyan-300 bg-cyan-950/30 px-2 py-1 rounded">
                                                    {cmd}
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>
                    )}

                    <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl p-4 flex flex-col gap-3 overflow-y-auto max-h-[400px] custom-scrollbar">
                        <h4 className="font-bold text-white text-sm flex items-center gap-2">
                            <svg className="w-4 h-4 text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            Scripting Reference
                        </h4>

                        <div className="space-y-4">
                            <div>
                                <h5 className="text-xs font-bold text-[var(--accent-primary)] mb-1">Entry Points</h5>
                                <ul className="text-xs text-[var(--text-secondary)] space-y-1 font-mono">
                                    <li><span className="text-white">fn on_load()</span> - Before playback (Transformer)</li>
                                    <li><span className="text-white">fn on_unload()</span> - After playback (Transformer)</li>
                                    <li><span className="text-white">fn load_content(params)</span> - Fetch content (Loader)</li>
                                </ul>
                            </div>

                            <div>
                                <h5 className="text-xs font-bold text-[var(--accent-primary)] mb-1">Variables</h5>
                                <ul className="text-xs text-[var(--text-secondary)] space-y-1 font-mono">
                                    <li><span className="text-white">params</span> - Input JSON object</li>
                                    <li><span className="text-white">settings</span> - Global Settings Map</li>
                                    <li><span className="text-white">dj</span> - Current DJ Profile (Transformer)</li>
                                    <li><span className="text-white">content_item</span> - Target Content (Transformer)</li>
                                </ul>
                            </div>

                            <div>
                                <h5 className="text-xs font-bold text-[var(--accent-primary)] mb-1">Common Functions</h5>
                                <ul className="text-xs text-[var(--text-secondary)] space-y-1 font-mono">
                                    <li>print(msg)</li>
                                    <li>to_json(val)</li>
                                </ul>
                            </div>

                            <div>
                                <h5 className="text-xs font-bold text-[var(--accent-primary)] mb-1">Server (Context/DJ)</h5>
                                <ul className="text-xs text-[var(--text-secondary)] space-y-1 font-mono">
                                    <li>http_get(url)</li>
                                    <li>parse_xml(xml)</li>
                                    <li>get_time(fmt, tz)</li>
                                    <li>log_info(msg)</li>
                                </ul>
                            </div>

                            <div>
                                <h5 className="text-xs font-bold text-[var(--accent-primary)] mb-1">Node (Loader/Overlay)</h5>
                                <ul className="text-xs text-[var(--text-secondary)] space-y-1 font-mono">
                                    <li>shell_execute(cmd)</li>
                                    <li>download_file(url, path)</li>
                                    <li>mpv_send(cmd)</li>
                                    <li>get_env(key)</li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}
