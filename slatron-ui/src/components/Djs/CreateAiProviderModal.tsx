import { useEffect, useState } from 'react'
import { useDjStore } from '../../stores/djStore'
import { AiProvider } from '../../api/dj'
import { apiClient } from '../../api/client'

interface SystemCapabilities {
    orpheus_enabled: boolean
}

interface CreateAiProviderModalProps {
    isOpen: boolean
    onClose: () => void
    initialData?: AiProvider | null
}

export default function CreateAiProviderModal({ isOpen, onClose, initialData }: CreateAiProviderModalProps) {
    const { addAiProvider, updateAiProvider } = useDjStore()
    const [formData, setFormData] = useState({
        name: '',
        provider_category: 'llm', // 'llm' or 'tts'
        provider_type: 'openai',
        endpoint_url: '',
        api_key: '',
        model_name: '',
        is_active: true
    })
    const [isSubmitting, setIsSubmitting] = useState(false)
    const [capabilities, setCapabilities] = useState<SystemCapabilities | null>(null)

    useEffect(() => {
        apiClient.get<SystemCapabilities>('/api/system/capabilities')
            .then(res => setCapabilities(res.data))
            .catch(err => console.error("Failed to fetch capabilities", err))
    }, [])

    useEffect(() => {
        if (isOpen) {
            if (initialData) {
                setFormData({
                    name: initialData.name,
                    provider_category: initialData.provider_category || 'llm',
                    provider_type: initialData.provider_type,
                    endpoint_url: initialData.endpoint_url || '',
                    api_key: '', // Don't show API key for security, user can overwrite if needed
                    model_name: initialData.model_name || '',
                    is_active: initialData.is_active
                })
            } else {
                // Reset for new entry
                setFormData({
                    name: '',
                    provider_category: 'llm',
                    provider_type: 'openai',
                    endpoint_url: '',
                    api_key: '',
                    model_name: '',
                    is_active: true
                })
            }
        }
    }, [isOpen, initialData])

    // Reset provider type when category changes
    const handleCategoryChange = (category: string) => {
        setFormData({
            ...formData,
            provider_category: category,
            provider_type: category === 'llm' ? 'openai' : 'orpheus',
            // Reset fields that might not apply
            endpoint_url: '',
            model_name: ''
        })
    }

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        setIsSubmitting(true)
        try {
            const payload = {
                ...formData,
                endpoint_url: formData.endpoint_url || undefined,
                api_key: formData.api_key || undefined,
                model_name: formData.model_name || undefined
            }

            if (initialData) {
                await updateAiProvider(initialData.id, payload)
            } else {
                await addAiProvider(payload)
            }
            onClose()
        } catch (e) {
            console.error(e)
            alert('Failed to save provider: ' + (e as Error).message)
        } finally {
            setIsSubmitting(false)
        }
    }

    if (!isOpen) return null

    const isLlm = formData.provider_category === 'llm';
    const isCustomLlm = formData.provider_type === 'custom_llm' || formData.provider_type === 'ollama' || formData.provider_type === 'lmstudio';
    const isOrpheus = formData.provider_type === 'orpheus';

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl w-full max-w-lg shadow-2xl animate-fade-in">
                <div className="p-6">
                    <h2 className="text-xl font-bold text-white mb-6">
                        {initialData ? 'Edit AI Provider' : 'Add AI Provider'}
                    </h2>

                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div>
                            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Name</label>
                            <input
                                type="text"
                                required
                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                value={formData.name}
                                onChange={e => setFormData({ ...formData, name: e.target.value })}
                                placeholder="My Provider"
                            />
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Category</label>
                                <select
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                    value={formData.provider_category}
                                    onChange={e => handleCategoryChange(e.target.value)}
                                >
                                    <option value="llm">Text Generation (LLM)</option>
                                    <option value="tts">Voice Synthesis (TTS)</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Provider Service</label>
                                <select
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none"
                                    value={formData.provider_type}
                                    onChange={e => setFormData({ ...formData, provider_type: e.target.value })}
                                >
                                    {isLlm ? (
                                        <>
                                            <option value="openai">OpenAI</option>
                                            <option value="anthropic">Anthropic</option>
                                            <option value="gemini">Google Gemini</option>
                                            <option value="ollama">Ollama</option>
                                            <option value="lmstudio">LM Studio</option>
                                            <option value="custom_llm">Custom (OpenAI Compatible)</option>
                                        </>
                                    ) : (
                                        <>
                                            <option
                                                value="orpheus"
                                                disabled={!capabilities?.orpheus_enabled}
                                            >
                                                Orpheus (Local) {!capabilities?.orpheus_enabled && '(Requires ML Feature)'}
                                            </option>
                                            <option value="gemini-tts">Google Gemini (TTS)</option>
                                            <option value="elevenlabs" disabled>ElevenLabs (Coming Soon)</option>
                                        </>
                                    )}
                                </select>
                            </div>
                        </div>

                        {/* Endpoint URL: Required for Custom LLM, Ollama, LMStudio, Orpheus */}
                        {(isCustomLlm || isOrpheus) && (
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                                    {isOrpheus ? 'LM Studio URL (for Orpheus)' : 'API Endpoint URL'}
                                </label>
                                <input
                                    type="text"
                                    placeholder={
                                        formData.provider_type === 'ollama' ? 'http://localhost:11434/api/generate' :
                                            formData.provider_type === 'orpheus' ? 'http://127.0.0.1:1234/v1/completions' :
                                                'https://api.openai.com/v1/chat/completions'
                                    }
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none font-mono"
                                    value={formData.endpoint_url}
                                    onChange={e => setFormData({ ...formData, endpoint_url: e.target.value })}
                                />
                                <p className="text-xs text-[var(--text-tertiary)] mt-1">
                                    {isOrpheus
                                        ? "Point this to a local LLM server (like LM Studio) running the Orpheus model."
                                        : "Full URL to the chat completions endpoint."}
                                </p>
                            </div>
                        )}

                        {/* API Key: Required for everything EXCEPT internal/local if not needed (Ollama usually no key, but others yes) */}
                        {/* Actually Ollama usually no key. Orpheus usually no key if local. */}
                        {/* OpenAI, Anthropic, Gemini, DeepSeek etc need keys. */}
                        {/* Custom LLM might need key. */}
                        {(!isOrpheus && formData.provider_type !== 'ollama') && (
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                                    API Key {initialData && '(Leave blank to keep unchanged)'}
                                </label>
                                <input
                                    type="password"
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none font-mono"
                                    value={formData.api_key}
                                    onChange={e => setFormData({ ...formData, api_key: e.target.value })}
                                    placeholder="sk-..."
                                />
                            </div>
                        )}

                        {/* Model Name: Required for Custom LLM, Ollama. Optional/Predefined for others. */}
                        {/* Orpheus doesn't need 'model name' distinct from voice usually, but code uses 'voice' arg. */}
                        {/* For LLMS, we often want to specify model (gpt-4, claude-3, etc). */}
                        {isLlm && (
                            <div>
                                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                                    Model Name (Optional)
                                </label>
                                <input
                                    type="text"
                                    placeholder={
                                        formData.provider_type === 'ollama' ? 'llama3' :
                                            formData.provider_type === 'anthropic' ? 'claude-3-opus-20240229' :
                                                formData.provider_type === 'gemini' ? 'gemini-pro' :
                                                    'gpt-4o'
                                    }
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2.5 text-white text-sm focus:border-indigo-500 outline-none font-mono"
                                    value={formData.model_name}
                                    onChange={e => setFormData({ ...formData, model_name: e.target.value })}
                                />
                            </div>
                        )}

                        <div className="flex items-center gap-2 pt-2">
                            <input
                                type="checkbox"
                                id="is_active"
                                checked={formData.is_active}
                                onChange={e => setFormData({ ...formData, is_active: e.target.checked })}
                                className="w-4 h-4 rounded border-gray-600 text-indigo-600 focus:ring-indigo-500"
                            />
                            <label htmlFor="is_active" className="text-sm font-medium text-white">Active</label>
                        </div>

                        <div className="flex justify-end gap-3 pt-4 border-t border-[var(--border-color)]">
                            <button
                                type="button"
                                onClick={onClose}
                                className="px-4 py-2 text-sm text-[var(--text-secondary)] hover:text-white transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                type="submit"
                                disabled={isSubmitting}
                                className="btn-primary"
                            >
                                {isSubmitting ? 'Saving...' : (initialData ? 'Save Changes' : 'Create Provider')}
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    )
}
