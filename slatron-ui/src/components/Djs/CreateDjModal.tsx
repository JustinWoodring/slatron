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
import {
    DndContext,
    closestCenter,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    DragEndEvent,
} from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    useSortable,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

interface CreateDjModalProps {
    isOpen: boolean
    onClose: () => void
    onDjAdded: () => void
    initialDj?: DjProfile
}

interface ScriptConfig {
    id: number
    params: Record<string, any>
}

// Sortable Item Component
const SortableScriptItem = ({
    script,
    onRemove,
    renderParams
}: {
    script: Script,
    config: ScriptConfig,
    onRemove: (id: number) => void,
    renderParams: (script: Script) => React.ReactNode
}) => {
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
    } = useSortable({ id: script.id });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
    };

    return (
        <div ref={setNodeRef} style={style} className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded mb-2 group">
            <div className="flex items-center p-3">
                {/* Drag Handle */}
                <div {...attributes} {...listeners} className="mr-3 cursor-grab text-[var(--text-secondary)] hover:text-white">
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                    </svg>
                </div>

                <div className="flex-1">
                    <div className="font-medium text-white">{script.name}</div>
                    {script.description && <div className="text-xs text-[var(--text-secondary)]">{script.description}</div>}
                </div>

                <button
                    onClick={() => onRemove(script.id)}
                    className="ml-3 text-[var(--text-secondary)] hover:text-red-400 p-1"
                    title="Remove script"
                >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
            {renderParams(script)}
        </div>
    );
}

export default function CreateDjModal({ isOpen, onClose, onDjAdded, initialDj }: CreateDjModalProps) {
    const [activeTab, setActiveTab] = useState<'profile' | 'scripts' | 'memories'>('profile')
    // We keep state separate for complex structures usually, but keeping unified is fine if typed well
    const [name, setName] = useState('')
    const [personalityPrompt, setPersonalityPrompt] = useState('')
    const [voiceConfigJson, setVoiceConfigJson] = useState('{}')
    const [contextDepth, setContextDepth] = useState(5)
    // The core change: proper script config array
    const [selectedScripts, setSelectedScripts] = useState<ScriptConfig[]>([])

    const [voiceProviderId, setVoiceProviderId] = useState<string>('')
    const [llmProviderId, setLlmProviderId] = useState<string>('')
    const [talkativeness, setTalkativeness] = useState(1.0)

    const [contextScripts, setContextScripts] = useState<Script[]>([])
    const [providers, setProviders] = useState<AiProvider[]>([])
    const [memories, setMemories] = useState<DjMemory[]>([])
    const [isMemoriesLoading, setIsMemoriesLoading] = useState(false)
    const [newMemoryContent, setNewMemoryContent] = useState('')
    const [newMemoryImportance, setNewMemoryImportance] = useState(5)
    const [isAddingMemory, setIsAddingMemory] = useState(false)
    const [isSubmitting, setIsSubmitting] = useState(false)

    // DnD Sensors
    const sensors = useSensors(
        useSensor(PointerSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        })
    );

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        if (active.id !== over?.id) {
            setSelectedScripts((items) => {
                const oldIndex = items.findIndex(i => i.id === active.id);
                const newIndex = items.findIndex(i => i.id === over?.id);
                return arrayMove(items, oldIndex, newIndex);
            });
        }
    };

    useEffect(() => {
        if (isOpen) {
            loadScripts();
            loadProviders();
            if (initialDj) {
                setName(initialDj.name);
                setPersonalityPrompt(initialDj.personality_prompt);
                setVoiceConfigJson(initialDj.voice_config_json);
                setContextDepth(initialDj.context_depth);
                setVoiceProviderId(initialDj.voice_provider_id?.toString() || '');
                setLlmProviderId(initialDj.llm_provider_id?.toString() || '');
                setTalkativeness(initialDj.talkativeness ?? 1.0);

                // Parse Scripts
                if (initialDj.context_script_ids) {
                    const raw = initialDj.context_script_ids.trim();
                    if (raw.startsWith('[')) {
                        try {
                            setSelectedScripts(JSON.parse(raw));
                        } catch (e) {
                            console.error("Failed to parse script JSON", e);
                            setSelectedScripts([]);
                        }
                    } else {
                        // Legacy CSV
                        const ids = raw.split(',').map(s => parseInt(s.trim())).filter(n => !isNaN(n));
                        setSelectedScripts(ids.map(id => ({ id, params: {} })));
                    }
                } else {
                    setSelectedScripts([]);
                }

                loadMemories(initialDj.id);
            } else {
                // Reset defaults
                setName('');
                setPersonalityPrompt('');
                setVoiceConfigJson(JSON.stringify({
                    stability: 0.5,
                    similarity_boost: 0.75
                }, null, 2));
                setContextDepth(5);
                setSelectedScripts([]);
                setVoiceProviderId('');
                setLlmProviderId('');
                setTalkativeness(1.0);
                setMemories([]);
            }
            setActiveTab('profile');
        }
    }, [initialDj, isOpen])

    const loadScripts = async () => {
        try {
            const scripts = await getScripts();
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

        // Serialize Scripts to JSON string
        // If empty, null.
        const serializedScripts = selectedScripts.length > 0
            ? JSON.stringify(selectedScripts)
            : null;

        const payload: NewDjProfile = {
            name,
            personality_prompt: personalityPrompt,
            voice_config_json: voiceConfigJson,
            context_depth: contextDepth,
            context_script_ids: serializedScripts,
            voice_provider_id: voiceProviderId ? Number(voiceProviderId) : null,
            llm_provider_id: llmProviderId ? Number(llmProviderId) : null,
            talkativeness,
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

    const addScript = (id: number) => {
        if (!selectedScripts.some(s => s.id === id)) {
            setSelectedScripts([...selectedScripts, { id, params: {} }])
        }
    }

    const removeScript = (id: number) => {
        setSelectedScripts(selectedScripts.filter(s => s.id !== id))
    }

    const updateScriptParam = (scriptId: number, paramKey: string, value: any) => {
        setSelectedScripts(prev => prev.map(s => {
            if (s.id === scriptId) {
                return { ...s, params: { ...s.params, [paramKey]: value } }
            }
            return s;
        }));
    }

    const renderScriptParams = (script: Script) => {
        if (!script.parameters_schema) return null;

        let schema: Record<string, string>;
        try {
            schema = JSON.parse(script.parameters_schema);
        } catch (e) {
            return <div className="text-red-500 text-xs mt-2">Invalid Schema JSON</div>;
        }

        const config = selectedScripts.find(s => s.id === script.id);
        if (!config) return null;

        return (
            <div className="mx-3 mb-3 p-3 bg-[var(--bg-tertiary)] rounded border border-[var(--border-color)] text-sm">
                <div className="text-[var(--text-secondary)] text-xs mb-2 uppercase tracking-wider font-bold">Parameters</div>
                <div className="space-y-3">
                    {Object.entries(schema).map(([key, type]) => (
                        <div key={key}>
                            <label className="block text-[var(--text-secondary)] text-xs mb-1 capitalize">{key}</label>
                            <input
                                type={type === 'number' ? 'number' : 'text'}
                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-1.5 text-white text-xs focus:border-indigo-500 focus:outline-none"
                                placeholder={`Enter ${key}...`}
                                value={config.params[key] || ''}
                                onChange={(e) => updateScriptParam(script.id, key, e.target.value)}
                            />
                        </div>
                    ))}
                </div>
            </div>
        )
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
                                    value={name}
                                    onChange={e => setName(e.target.value)}
                                />
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Personality Prompt</label>
                                <textarea
                                    required
                                    rows={4}
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded p-2 text-white text-sm focus:border-indigo-500 focus:outline-none"
                                    value={personalityPrompt}
                                    onChange={e => setPersonalityPrompt(e.target.value)}
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
                                        value={llmProviderId}
                                        onChange={e => setLlmProviderId(e.target.value)}
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
                                        value={voiceProviderId}
                                        onChange={e => setVoiceProviderId(e.target.value)}
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
                                    value={voiceConfigJson}
                                    onChange={e => setVoiceConfigJson(e.target.value)}
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
                                    value={contextDepth}
                                    onChange={e => setContextDepth(parseInt(e.target.value))}
                                />
                                <p className="text-xs text-[var(--text-secondary)] mt-1">
                                    Number of previous tracks/events to include in the AI context window.
                                </p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                                    Talkativeness ({Math.round(talkativeness * 100)}%)
                                </label>
                                <input
                                    type="range"
                                    min="0"
                                    max="1"
                                    step="0.05"
                                    className="w-full"
                                    value={talkativeness}
                                    onChange={e => setTalkativeness(parseFloat(e.target.value))}
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
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-2">Selected Scripts (Drag to Order)</label>

                                <DndContext
                                    sensors={sensors}
                                    collisionDetection={closestCenter}
                                    onDragEnd={handleDragEnd}
                                >
                                    <div className="min-h-[100px] mb-6 space-y-2">
                                        <SortableContext
                                            items={selectedScripts.map(s => s.id)}
                                            strategy={verticalListSortingStrategy}
                                        >
                                            {selectedScripts.length === 0 && (
                                                <div className="text-sm text-[var(--text-secondary)] italic border border-dashed border-[var(--border-color)] rounded p-4 text-center">
                                                    No scripts selected. Add from list below.
                                                </div>
                                            )}
                                            {selectedScripts.map(config => {
                                                const script = contextScripts.find(s => s.id === config.id);
                                                if (!script) return null;
                                                return (
                                                    <SortableScriptItem
                                                        key={config.id}
                                                        script={script}
                                                        config={config}
                                                        onRemove={removeScript}
                                                        renderParams={renderScriptParams}
                                                    />
                                                );
                                            })}
                                        </SortableContext>
                                    </div>
                                </DndContext>

                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-2">Available Scripts</label>
                                <div className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white text-sm max-h-[300px] overflow-y-auto">
                                    {contextScripts.length === 0 && (
                                        <div className="p-2.5 text-[var(--text-secondary)] italic">No server context scripts available.</div>
                                    )}
                                    {contextScripts
                                        .filter(s => !selectedScripts.some(sel => sel.id === s.id))
                                        .map(s => (
                                            <div
                                                key={s.id}
                                                className="flex items-center justify-between p-3 border-b border-[var(--border-color)] last:border-0 hover:bg-[var(--bg-hover)] transition-colors cursor-pointer group"
                                                onClick={() => addScript(s.id)}
                                            >
                                                <div>
                                                    <div className="font-medium text-white">{s.name}</div>
                                                    {s.description && <div className="text-xs text-[var(--text-secondary)]">{s.description}</div>}
                                                </div>
                                                <button className="text-indigo-400 group-hover:text-white bg-[var(--bg-secondary)] group-hover:bg-indigo-600 w-6 h-6 rounded flex items-center justify-center transition-colors">
                                                    +
                                                </button>
                                            </div>
                                        ))}
                                </div>
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

