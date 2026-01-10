import { useEffect, useState } from 'react';
import { apiClient } from '../../api/client';
import { useDjStore } from '../../stores/djStore';

interface GlobalSetting {
    key: string;
    value: string;
    description?: string;
}

export default function OnboardingWizard() {
    const [isOpen, setIsOpen] = useState(false);
    const [step, setStep] = useState(1);
    const [settings, setSettings] = useState<GlobalSetting[]>([]);

    // Store Actions
    const { addAiProvider, addDj } = useDjStore();

    // Form State
    const [stationName, setStationName] = useState("Slatron TV");
    const [timezone, setTimezone] = useState("America/Chicago");
    const [error, setError] = useState<string | null>(null);
    const [isSaving, setIsSaving] = useState(false);

    // AI / DJ State
    const [llmProvider, setLlmProvider] = useState('openai');
    const [llmKey, setLlmKey] = useState('');
    const [llmEndpoint, setLlmEndpoint] = useState('');
    const [llmModel, setLlmModel] = useState('');
    const [skipLlm, setSkipLlm] = useState(false);

    const [ttsProvider, setTtsProvider] = useState('gemini-tts');
    const [ttsKey, setTtsKey] = useState('');
    const [ttsEndpoint, setTtsEndpoint] = useState('');
    const [skipTts, setSkipTts] = useState(false);

    const [djName, setDjName] = useState('DJ Synapse');
    const [skipDj, setSkipDj] = useState(false);

    useEffect(() => {
        fetchSettings();
    }, []);

    const fetchSettings = async () => {
        try {
            const res = await apiClient.get<GlobalSetting[]>('/api/settings');
            const fetched = res.data;
            setSettings(fetched);

            const complete = fetched.find(s => s.key === 'onboarding_complete');
            if (!complete || complete.value === 'false') {
                setIsOpen(true);
                const name = fetched.find(s => s.key === 'station_name');
                if (name) setStationName(name.value);
                const tz = fetched.find(s => s.key === 'timezone');
                if (tz) setTimezone(tz.value);
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleSaveSetting = async (key: string, value: string) => {
        try {
            await apiClient.put(`/api/settings/${key}`, {
                key,
                value,
                description: settings.find(s => s.key === key)?.description || "Updated via Onboarding"
            });
        } catch (err) {
            console.error("Failed to save " + key, err);
            throw err;
        }
    };

    const handleNext = () => {
        setError(null);
        if (step === 1) {
            if (!stationName.trim()) return setError("Station Name is required");
            setStep(2);
        } else if (step === 2) {
            setStep(3);
        } else if (step === 3) {
            // LLM
            if (!skipLlm && !llmKey.trim() && llmProvider !== 'ollama') {
                // allow proceeding but maybe warn? For now assume user might want to fill later from UI
            }
            setStep(4);
        } else if (step === 4) {
            // TTS
            setStep(5);
        } else if (step === 5) {
            // DJ
            if (!skipDj && !djName.trim()) return setError("DJ Name is required");
            setStep(6);
        }
    };

    const handleFinish = async () => {
        setIsSaving(true);
        setError(null);
        try {
            // Save Base Settings
            await handleSaveSetting('station_name', stationName);
            await handleSaveSetting('timezone', timezone);

            // Create LLM
            if (!skipLlm && (llmKey || ['ollama', 'lmstudio', 'custom_llm'].includes(llmProvider))) {
                // Defaults if empty
                let finalEndpoint = llmEndpoint;
                let finalModel = llmModel;

                if (!finalEndpoint) {
                    if (llmProvider === 'ollama') finalEndpoint = 'http://localhost:11434/api/generate';
                    if (llmProvider === 'lmstudio') finalEndpoint = 'http://localhost:1234/v1/chat/completions';
                }
                if (!finalModel) {
                    if (llmProvider === 'ollama') finalModel = 'llama3';
                }

                await addAiProvider({
                    name: 'Primary LLM',
                    provider_category: 'llm',
                    provider_type: llmProvider,
                    api_key: llmKey,
                    endpoint_url: finalEndpoint,
                    model_name: finalModel,
                    is_active: true
                });
            }

            // Create TTS
            if (!skipTts && (ttsKey || ttsProvider === 'orpheus')) {
                // Default endpoint for Orpheus TTS
                let finalTtsEndpoint = ttsEndpoint;
                if (!finalTtsEndpoint && ttsProvider === 'orpheus') {
                    finalTtsEndpoint = 'http://127.0.0.1:1234/v1/completions';
                }

                await addAiProvider({
                    name: 'Primary TTS',
                    provider_category: 'tts',
                    provider_type: ttsProvider,
                    api_key: ttsKey,
                    endpoint_url: finalTtsEndpoint,
                    is_active: true
                });
            }

            // Create DJ
            if (!skipDj && djName) {
                await addDj({
                    name: djName,
                    personality_prompt: "You are a professional yet witty radio host. Introduce tracks with style.",
                    voice_config_json: "{}",
                    talkativeness: 0.8,
                    context_depth: 5,
                });
            }

            // Mark Complete
            await handleSaveSetting('onboarding_complete', 'true');
            setIsOpen(false);
            window.location.reload(); // Refresh to apply changes globally
        } catch (err) {
            console.error(err);
            setError("Failed to save configuration: " + (err as Error).message);
            setIsSaving(false);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm animate-fade-in">
            <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[90vh]">

                {/* Header */}
                <div className="p-6 border-b border-[var(--border-color)] bg-[var(--bg-tertiary)]/30 text-center">
                    <h2 className="text-xl font-bold text-white mb-2">Welcome to Slatron</h2>
                    <p className="text-[var(--text-secondary)]">Let's get your station on air.</p>
                </div>

                {/* Content */}
                <div className="p-8 space-y-6 flex-1 overflow-y-auto">
                    {error && (
                        <div className="bg-red-500/10 border border-red-500/20 text-red-200 p-3 rounded-lg text-sm mb-4">
                            {error}
                        </div>
                    )}

                    {step === 1 && (
                        <div className="space-y-4 animate-fade-in">
                            <h3 className="text-lg font-medium text-white">1. Name your Station</h3>
                            <p className="text-sm text-[var(--text-secondary)]">What should we call this broadcast facility?</p>
                            <input
                                type="text"
                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white focus:border-[var(--accent-primary)] outline-none text-lg"
                                placeholder="e.g. Channel 4 News"
                                value={stationName}
                                onChange={e => setStationName(e.target.value)}
                                autoFocus
                            />
                        </div>
                    )}

                    {step === 2 && (
                        <div className="space-y-4 animate-fade-in">
                            <h3 className="text-lg font-medium text-white">2. Set Timezone</h3>
                            <p className="text-sm text-[var(--text-secondary)]">This ensures your schedules run at the correct local time.</p>
                            <select
                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white focus:border-[var(--accent-primary)] outline-none"
                                value={timezone}
                                onChange={e => setTimezone(e.target.value)}
                            >
                                {(Intl as any).supportedValuesOf('timeZone').map((tz: string) => (
                                    <option key={tz} value={tz}>{tz}</option>
                                ))}
                            </select>
                        </div>
                    )}

                    {step === 3 && (
                        <div className="space-y-4 animate-fade-in">
                            <div className="flex justify-between items-center">
                                <h3 className="text-lg font-medium text-white">3. Setup Intelligence</h3>
                                <button className="text-xs text-[var(--text-tertiary)] hover:text-white" onClick={() => setSkipLlm(!skipLlm)}>
                                    {skipLlm ? "Enable Setup" : "Skip for now"}
                                </button>
                            </div>

                            {!skipLlm ? (
                                <>
                                    <p className="text-sm text-[var(--text-secondary)]">Connect an LLM provider to power your DJs.</p>
                                    <div>
                                        <label className="text-xs text-[var(--text-tertiary)] block mb-1">Provider</label>
                                        <select
                                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none"
                                            value={llmProvider}
                                            onChange={e => {
                                                const val = e.target.value;
                                                setLlmProvider(val);
                                                // Reset defaults on change
                                                if (val === 'ollama') { setLlmEndpoint('http://localhost:11434/api/generate'); setLlmModel('llama3'); }
                                                else if (val === 'lmstudio') { setLlmEndpoint('http://localhost:1234/v1/chat/completions'); setLlmModel(''); }
                                                else { setLlmEndpoint(''); setLlmModel(''); }
                                            }}
                                        >
                                            <option value="openai">OpenAI</option>
                                            <option value="anthropic">Anthropic</option>
                                            <option value="gemini">Google Gemini</option>
                                            <option value="ollama">Ollama (Local)</option>
                                            {/* Orpheus removed from LLM options */}
                                            <option value="lmstudio">LM Studio (Local)</option>
                                            <option value="custom_llm">Custom / OpenAI Compatible</option>
                                        </select>
                                    </div>

                                    {['ollama', 'lmstudio', 'custom_llm'].includes(llmProvider) && (
                                        <>
                                            <div>
                                                <label className="text-xs text-[var(--text-tertiary)] block mb-1">Endpoint URL</label>
                                                <input
                                                    type="text"
                                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none font-mono text-sm"
                                                    placeholder="http://..."
                                                    value={llmEndpoint}
                                                    onChange={e => setLlmEndpoint(e.target.value)}
                                                />
                                            </div>
                                            <div>
                                                <label className="text-xs text-[var(--text-tertiary)] block mb-1">Model Name {llmProvider === 'ollama' && '(e.g. llama3)'}</label>
                                                <input
                                                    type="text"
                                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none font-mono text-sm"
                                                    placeholder="Model ID"
                                                    value={llmModel}
                                                    onChange={e => setLlmModel(e.target.value)}
                                                />
                                            </div>
                                        </>
                                    )}

                                    {!['ollama'].includes(llmProvider) && (
                                        <div>
                                            <label className="text-xs text-[var(--text-tertiary)] block mb-1">API Key</label>
                                            <input
                                                type="password"
                                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none font-mono text-sm"
                                                placeholder="sk-..."
                                                value={llmKey}
                                                onChange={e => setLlmKey(e.target.value)}
                                            />
                                        </div>
                                    )}
                                </>
                            ) : (
                                <div className="p-4 bg-[var(--bg-primary)] rounded-lg text-center text-[var(--text-tertiary)] text-sm italic">
                                    You can configure this later in settings.
                                </div>
                            )}
                        </div>
                    )}

                    {step === 4 && (
                        <div className="space-y-4 animate-fade-in">
                            <div className="flex justify-between items-center">
                                <h3 className="text-lg font-medium text-white">4. Setup Voice</h3>
                                <button className="text-xs text-[var(--text-tertiary)] hover:text-white" onClick={() => setSkipTts(!skipTts)}>
                                    {skipTts ? "Enable Setup" : "Skip for now"}
                                </button>
                            </div>

                            {!skipTts ? (
                                <>
                                    <p className="text-sm text-[var(--text-secondary)]">Choose a text-to-speech engine for your DJs.</p>
                                    <div>
                                        <label className="text-xs text-[var(--text-tertiary)] block mb-1">Provider</label>
                                        <select
                                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none"
                                            value={ttsProvider}
                                            onChange={e => {
                                                const val = e.target.value;
                                                setTtsProvider(val);
                                                if (val === 'orpheus') setTtsEndpoint('http://127.0.0.1:1234/v1/completions');
                                                else setTtsEndpoint('');
                                            }}
                                        >
                                            <option value="gemini-tts">Google Gemini TTS</option>
                                            <option value="orpheus">Orpheus (Local)</option>
                                            {/* Add others if supported easily */}
                                        </select>
                                    </div>

                                    {ttsProvider === 'orpheus' && (
                                        <div>
                                            <label className="text-xs text-[var(--text-tertiary)] block mb-1">LM Studio Endpoint (for Orpheus)</label>
                                            <input
                                                type="text"
                                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none font-mono text-sm"
                                                placeholder="http://127.0.0.1:1234/v1/completions"
                                                value={ttsEndpoint}
                                                onChange={e => setTtsEndpoint(e.target.value)}
                                            />
                                        </div>
                                    )}

                                    {ttsProvider !== 'orpheus' && (
                                        <div>
                                            <label className="text-xs text-[var(--text-tertiary)] block mb-1">API Key</label>
                                            <input
                                                type="password"
                                                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none font-mono text-sm"
                                                placeholder="AI Key"
                                                value={ttsKey}
                                                onChange={e => setTtsKey(e.target.value)}
                                            />
                                        </div>
                                    )}
                                </>
                            ) : (
                                <div className="p-4 bg-[var(--bg-primary)] rounded-lg text-center text-[var(--text-tertiary)] text-sm italic">
                                    You can configure this later in settings.
                                </div>
                            )}
                        </div>
                    )}

                    {step === 5 && (
                        <div className="space-y-4 animate-fade-in">
                            <div className="flex justify-between items-center">
                                <h3 className="text-lg font-medium text-white">5. Meet your DJ</h3>
                                <button className="text-xs text-[var(--text-tertiary)] hover:text-white" onClick={() => setSkipDj(!skipDj)}>
                                    {skipDj ? "Enable Setup" : "Skip for now"}
                                </button>
                            </div>

                            {!skipDj ? (
                                <>
                                    <p className="text-sm text-[var(--text-secondary)]">Give your first AI personality a name.</p>
                                    <input
                                        type="text"
                                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 text-white outline-none text-lg"
                                        placeholder="DJ Name"
                                        value={djName}
                                        onChange={e => setDjName(e.target.value)}
                                        autoFocus
                                    />
                                    <p className="text-xs text-[var(--text-tertiary)]">We'll set them up with a default personality and voice.</p>
                                </>
                            ) : (
                                <div className="p-4 bg-[var(--bg-primary)] rounded-lg text-center text-[var(--text-tertiary)] text-sm italic">
                                    You can create DJs later.
                                </div>
                            )}
                        </div>
                    )}

                    {step === 6 && (
                        <div className="space-y-4 animate-fade-in">
                            <h3 className="text-lg font-medium text-white">6. Ready to Launch?</h3>
                            <div className="bg-[var(--bg-primary)] rounded-lg p-4 space-y-2 text-sm">
                                <div className="flex justify-between">
                                    <span className="text-[var(--text-secondary)]">Station:</span>
                                    <span className="text-white font-medium">{stationName}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-[var(--text-secondary)]">Timezone:</span>
                                    <span className="text-white font-medium">{timezone}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-[var(--text-secondary)]">LLM:</span>
                                    <span className="text-white font-medium">{skipLlm ? 'Skipped' : llmProvider}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-[var(--text-secondary)]">TTS:</span>
                                    <span className="text-white font-medium">{skipTts ? 'Skipped' : ttsProvider}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-[var(--text-secondary)]">DJ:</span>
                                    <span className="text-white font-medium">{skipDj ? 'Skipped' : djName}</span>
                                </div>
                            </div>
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="p-6 border-t border-[var(--border-color)] bg-[var(--bg-primary)] flex justify-between items-center">
                    <div className="flex gap-1">
                        {[1, 2, 3, 4, 5, 6].map(i => (
                            <div key={i} className={`h-2 w-2 rounded-full transition-colors ${step >= i ? 'bg-[var(--accent-primary)]' : 'bg-[var(--bg-tertiary)]'}`} />
                        ))}
                    </div>

                    <button
                        onClick={step === 6 ? handleFinish : handleNext}
                        disabled={isSaving}
                        className="px-6 py-2.5 bg-[var(--accent-primary)] hover:bg-[var(--accent-secondary)] text-white rounded-lg font-medium transition-all shadow-lg shadow-indigo-500/20 disabled:opacity-50"
                    >
                        {isSaving ? "Setting Up..." : (step === 6 ? "Launch Dashboard" : "Next Step")}
                    </button>
                </div>
            </div>
        </div>
    );
}
