import React, { useState, useEffect } from 'react'
import {
    createDj,
    updateDj,
    DjProfile,
    NewDjProfile,
    DjMemory,
    getDjMemories,
    createDjMemory,
    deleteDjMemory,
    getAiProviders,
    AiProvider,
} from '../../api/dj'
import { getScripts, Script } from '../../api/scripts'

interface CreateDjModalProps {
    isOpen: boolean
    onClose: () => void
    onDjAdded: () => void
    initialDj?: DjProfile
}

export default function CreateDjModal({ isOpen, onClose, onDjAdded, initialDj }: CreateDjModalProps) {
    const [activeTab, setActiveTab] = useState<'profile' | 'scripts' | 'memories'>('profile')
    const [formData, setFormData] = useState({
        name: '',
        personality_prompt: '',
        voice_config_json: '{}',
        context_depth: 5,
        context_script_ids_list: [] as number[],
        voice_provider_id: '' as string | number,
        llm_provider_id: '' as string | number,
        talkativeness: 1.0,
    })
    const [contextScripts, setContextScripts] = useState<Script[]>([])
    const [providers, setProviders] = useState<AiProvider[]>([])
    const [memories, setMemories] = useState<DjMemory[]>([])
    const [isMemoriesLoading, setIsMemoriesLoading] = useState(false)
    const [newMemoryContent, setNewMemoryContent] = useState('')
    const [newMemoryImportance, setNewMemoryImportance] = useState(5)
    const [isAddingMemory, setIsAddingMemory] = useState(false)
    const [isSubmitting, setIsSubmitting] = useState(false)

    useEffect(() => {
        if (isOpen) {
            loadScripts();
            loadProviders();
            if (initialDj) {
                setFormData({
                    name: initialDj.name,
                    personality_prompt: initialDj.personality_prompt,
                    voice_config_json: initialDj.voice_config_json,
                    context_depth: initialDj.context_depth,
                    context_script_ids_list: initialDj.context_script_ids
                        ? initialDj.context_script_ids.split(',').map(s => parseInt(s.trim())).filter(n => !isNaN(n))
                        : [],
                    voice_provider_id: initialDj.voice_provider_id || '',
                    llm_provider_id: initialDj.llm_provider_id || '',
                    talkativeness: initialDj.talkativeness ?? 1.0,
                });
                loadMemories(initialDj.id);
            } else {
                setFormData({
                    name: '',
                    personality_prompt: '',
                    voice_config_json: JSON.stringify({
                        stability: 0.5,
                        similarity_boost: 0.75
                    }, null, 2),
                    context_depth: 5,
                    context_script_ids_list: [] as number[],
                    voice_provider_id: '',
                    llm_provider_id: '',
                    talkativeness: 1.0,
                });
                setMemories([]);
            }
            setActiveTab('profile'); // Reset to profile on open
        }
    }, [initialDj, isOpen])

    const loadScripts = async () => {
        try {
            const scripts = await getScripts();
            // Filter only server_context scripts
            setContextScripts(scripts.filter(s => s.script_type === 'server_context'));
        } catch (error) {
            console.error("Failed to load scripts", error);
        }
    }

    const loadProviders = async () => {
        try {
            const data = await getAiProviders();
            setProviders(data.filter(p => p.is_active));
        } catch (error) {
            console.error("Failed to load providers", error);
        }
    }

    const loadMemories = async (djId: number) => {
        setIsMemoriesLoading(true);
        try {
            const mems = await getDjMemories(djId);
            setMemories(mems);
        } catch (error) {
            console.error("Failed to load memories", error);
        } finally {
            setIsMemoriesLoading(false);
        }
    }

    const handleSubmit = async (e?: React.FormEvent) => {
        if (e) e.preventDefault()

        const serializedScripts = formData.context_script_ids_list.length > 0
            ? formData.context_script_ids_list.join(',')
            : null;

        const payload: NewDjProfile = {
            name: formData.name,
            personality_prompt: formData.personality_prompt,
            voice_config_json: formData.voice_config_json,
            context_depth: formData.context_depth,
            context_script_ids: serializedScripts,
            voice_provider_id: formData.voice_provider_id ? Number(formData.voice_provider_id) : null,
            llm_provider_id: formData.llm_provider_id ? Number(formData.llm_provider_id) : null,
            talkativeness: formData.talkativeness,
        };

        try {
            setIsSubmitting(true)
            if (initialDj) {
                await updateDj(initialDj.id, payload)
            } else {
                await createDj(payload)
            }
            onDjAdded()
            onClose()
        } catch (error) {
            console.error('Failed to save DJ', error)
            alert('Failed to save DJ profile')
        } finally {
            setIsSubmitting(false)
        }
    }

    const handleAddMemory = async () => {
        if (!initialDj || !newMemoryContent.trim()) return;

        try {
            const newMem = await createDjMemory(initialDj.id, {
                dj_id: initialDj.id,
                memory_type: 'manual',
                content: newMemoryContent,
                importance_score: newMemoryImportance,
                happened_at: new Date().toISOString()
            });
            setMemories([newMem, ...memories]);
            setNewMemoryContent('');
            setNewMemoryImportance(5);
            setIsAddingMemory(false);
        } catch (error) {
            console.error("Failed to add memory", error);
            alert("Failed to add memory");
        }
    }

    const handleDeleteMemory = async (id: number) => {
        if (!confirm("Are you sure you want to delete this memory?")) return;
        try {
            await deleteDjMemory(id);
            setMemories(memories.filter(m => m.id !== id));
        } catch (error) {
            console.error("Failed to delete memory", error);
        }
    }

    const toggleScript = (id: number) => {
        setFormData(prev => {
            if (prev.context_script_ids_list.includes(id)) {
                return { ...prev, context_script_ids_list: prev.context_script_ids_list.filter(x => x !== id) }
            } else {
                return { ...prev, context_script_ids_list: [...prev.context_script_ids_list, id] }
            }
        });
    }

    if (!isOpen) return null

    return (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div className="bg-[var(--card-bg)] rounded-lg max-w-2xl w-full flex flex-col max-h-[90vh]">
                <div className="p-6 border-b border-[var(--border-color)]">
                    <h2 className="text-xl font-bold text-white">
                        {initialDj ? 'Edit DJ Profile' : 'Create New DJ'}
                    </h2>
                </div>

                <div className="flex border-b border-[var(--border-color)] px-6">
                    <button
                        className={`py-3 px-4 text-sm font-medium border-b-2 transition-colors ${activeTab === 'profile' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-[var(--text-secondary)] hover:text-white'} `}
                        onClick={() => setActiveTab('profile')}
                    >
                        Profile
                    </button>
                    <button
                        className={`py-3 px-4 text-sm font-medium border-b-2 transition-colors ${activeTab === 'scripts' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-[var(--text-secondary)] hover:text-white'} `}
                        onClick={() => setActiveTab('scripts')}
                    >
                        Context Scripts
                    </button>
                    {initialDj && (
                        <button
                            className={`py-3 px-4 text-sm font-medium border-b-2 transition-colors ${activeTab === 'memories' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-[var(--text-secondary)] hover:text-white'} `}
                            onClick={() => setActiveTab('memories')}
                        >
                            Memories
                        </button>
                    )}
                </div>

                <div className="p-6 overflow-y-auto flex-1">
                    {activeTab === 'profile' && (
                        <form id="dj-form" onSubmit={handleSubmit} className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Name</label>
                                <input
                                    type="text"
                                    required
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-white text-sm focus:border-indigo-500 focus:outline-none"
                                    value={formData.name}
                                    onChange={e => setFormData({ ...formData, name: e.target.value })}
                                />
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Personality Prompt</label>
                                <textarea
                                    required
                                    rows={4}
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-white text-sm focus:border-indigo-500 focus:outline-none"
                                    value={formData.personality_prompt}
                                    onChange={e => setFormData({ ...formData, personality_prompt: e.target.value })}
                                    placeholder="Describe the DJ's personality, style, and behavior..."
                                />
                                <p className="text-xs text-[var(--text-secondary)] mt-1">
                                    System prompt defining the persona. e.g. "You are a high-energy late night host."
                                </p>
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">LLM Provider</label>
                                    <select
                                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-white text-sm focus:border-indigo-500 focus:outline-none"
                                        value={formData.llm_provider_id}
                                        onChange={e => setFormData({ ...formData, llm_provider_id: e.target.value })}
                                    >
                                        <option value="">Default (Auto-Select)</option>
                                        {providers
                                            .filter(p => p.provider_category === 'llm')
                                            .map(p => (
                                                <option key={p.id} value={p.id}>
                                                    {p.name} ({p.provider_type})
                                                </option>
                                            ))}
                                    </select>
                                </div>
                                <div>
                                    <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Voice Provider</label>
                                    <select
                                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-white text-sm focus:border-indigo-500 focus:outline-none"
                                        value={formData.voice_provider_id}
                                        onChange={e => setFormData({ ...formData, voice_provider_id: e.target.value })}
                                    >
                                        <option value="">Default (Auto-Select)</option>
                                        {providers
                                            .filter(p => p.provider_category === 'tts')
                                            .map(p => (
                                                <option key={p.id} value={p.id}>
                                                    {p.name} ({p.provider_type})
                                                </option>
                                            ))}
                                    </select>
                                </div>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Voice Config (JSON)</label>
                                <textarea
                                    required
                                    rows={4}
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-white text-sm font-mono focus:border-indigo-500 focus:outline-none"
                                    value={formData.voice_config_json}
                                    onChange={e => setFormData({ ...formData, voice_config_json: e.target.value })}
                                />
                                <p className="text-xs text-[var(--text-secondary)] mt-1 font-mono">
                                    JSON settings for the TTS provider (e.g. stability, similarity).
                                </p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Context Depth (Messages)</label>
                                <input
                                    type="number"
                                    min="0"
                                    max="20"
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-white text-sm focus:border-indigo-500 focus:outline-none"
                                    value={formData.context_depth}
                                    onChange={e => setFormData({ ...formData, context_depth: parseInt(e.target.value) })}
                                />
                                <p className="text-xs text-[var(--text-secondary)] mt-1">
                                    Number of previous tracks/events to include in the AI context window.
                                </p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                                    Talkativeness ({Math.round(formData.talkativeness * 100)}%)
                                </label>
                                <input
                                    type="range"
                                    min="0"
                                    max="1"
                                    step="0.05"
                                    className="w-full"
                                    value={formData.talkativeness}
                                    onChange={e => setFormData({ ...formData, talkativeness: parseFloat(e.target.value) })}
                                />
                                <p className="text-xs text-[var(--text-secondary)] mt-1">
                                    Probability that the DJ will speak when triggered (vs just playing music).
                                </p>
                            </div>
                        </form>
                    )}

                    {activeTab === 'scripts' && (
                        <div className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Active Scripts</label>
                                <div className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white text-sm max-h-60 overflow-y-auto">
                                    {contextScripts.length === 0 && (
                                        <div className="p-2.5 text-[var(--text-secondary)] italic">No server context scripts available. Create one in the Scripts page.</div>
                                    )}
                                    {contextScripts.map(s => (
                                        <div
                                            key={s.id}
                                            onClick={() => toggleScript(s.id!)}
                                            className={`flex items-center p-3 cursor-pointer hover:bg-[var(--bg-hover)] transition-colors border-b border-[var(--border-color)] last:border-0 ${formData.context_script_ids_list.includes(s.id!) ? 'bg-indigo-900/20' : ''} `}
                                        >
                                            <div className={`w-5 h-5 mr-3 border rounded flex items-center justify-center transition-colors ${formData.context_script_ids_list.includes(s.id!) ? 'bg-indigo-500 border-indigo-500' : 'border-[var(--text-secondary)]'} `}>
                                                {formData.context_script_ids_list.includes(s.id!) && <span className="text-white text-xs font-bold">✓</span>}
                                            </div>
                                            <div>
                                                <div className="font-medium text-white">{s.name}</div>
                                                {s.description && <div className="text-xs text-[var(--text-secondary)]">{s.description}</div>}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                                <p className="text-xs text-[var(--text-secondary)] mt-2">
                                    Selected scripts will provide context to the AI during generation.
                                </p>
                            </div>
                        </div>
                    )}

                    {activeTab === 'memories' && initialDj && (
                        <div className="space-y-4">
                            <div className="flex justify-between items-center mb-2">
                                <h3 className="text-sm font-medium text-[var(--text-secondary)]">Long Term Memories</h3>
                                <button
                                    onClick={() => setIsAddingMemory(!isAddingMemory)}
                                    className="text-xs bg-indigo-600 hover:bg-indigo-700 text-white px-2 py-1 rounded transition-colors"
                                >
                                    {isAddingMemory ? 'Cancel' : '+ Add Memory'}
                                </button>
                            </div>

                            {isAddingMemory && (
                                <div className="bg-[var(--bg-primary)] p-3 rounded border border-[var(--border-color)] mb-4 animate-fade-in">
                                    <textarea
                                        className="w-full bg-[var(--card-bg)] border border-[var(--border-color)] rounded p-2 text-white text-sm mb-2 focus:border-indigo-500 focus:outline-none"
                                        rows={2}
                                        placeholder="e.g. Was a cheerleader in high school..."
                                        value={newMemoryContent}
                                        onChange={e => setNewMemoryContent(e.target.value)}
                                    />
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center">
                                            <span className="text-xs text-[var(--text-secondary)] mr-2">Importance (1-10):</span>
                                            <input
                                                type="number"
                                                min="1"
                                                max="10"
                                                value={newMemoryImportance}
                                                onChange={e => setNewMemoryImportance(parseInt(e.target.value))}
                                                className="w-12 bg-[var(--card-bg)] border border-[var(--border-color)] rounded p-1 text-white text-xs text-center"
                                            />
                                        </div>
                                        <button
                                            onClick={handleAddMemory}
                                            disabled={!newMemoryContent.trim()}
                                            className="text-xs bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white px-3 py-1 rounded"
                                        >
                                            Save
                                        </button>
                                    </div>
                                </div>
                            )}

                            {isMemoriesLoading ? (
                                <div className="text-center py-4 text-[var(--text-secondary)]">Loading memories...</div>
                            ) : memories.length === 0 ? (
                                <div className="text-center py-8 border border-dashed border-[var(--border-color)] rounded text-[var(--text-secondary)]">
                                    No memories recorded yet.
                                </div>
                            ) : (
                                <div className="space-y-2 max-h-60 overflow-y-auto pr-1">
                                    {memories.map(mem => (
                                        <div key={mem.id} className="bg-[var(--bg-primary)] p-3 rounded border border-[var(--border-color)] group hover:border-indigo-500/50 transition-colors">
                                            <div className="flex justify-between items-start mb-1">
                                                <div className="flex items-center gap-2">
                                                    <span className={`text-[10px] px-1.5 py-0.5 rounded ${mem.importance_score >= 8 ? 'bg-yellow-900/50 text-yellow-400 border border-yellow-700' : 'bg-gray-700 text-gray-300'} `}>
                                                        Imp: {mem.importance_score}
                                                    </span>
                                                    <span className="text-[10px] text-[var(--text-secondary)]">
                                                        {new Date(mem.created_at).toLocaleDateString()}
                                                    </span>
                                                </div>
                                                <button
                                                    onClick={() => handleDeleteMemory(mem.id)}
                                                    className="text-[var(--text-secondary)] hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
                                                    title="Delete memory"
                                                >
                                                    ✕
                                                </button>
                                            </div>
                                            <p className="text-sm text-gray-200">{mem.content}</p>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    )}
                </div>

                <div className="flex justify-end gap-3 pt-4 border-t border-[var(--border-color)] px-6 pb-6">
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-2 text-sm text-[var(--text-secondary)] hover:text-white transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={() => handleSubmit()}
                        disabled={isSubmitting}
                        className="btn-primary"
                    >
                        {isSubmitting ? 'Saving...' : (initialDj ? 'Save Changes' : 'Create DJ')}
                    </button>
                </div>
            </div>
        </div>
    )
}

