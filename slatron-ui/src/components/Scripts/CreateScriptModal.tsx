import React, { useState } from 'react'
import { useScriptStore } from '../../stores/scriptStore'
import { useNavigate } from 'react-router-dom'

interface CreateScriptModalProps {
    isOpen: boolean
    onClose: () => void
}

export const CreateScriptModal = ({ isOpen, onClose }: CreateScriptModalProps) => {
    const { createScript } = useScriptStore()
    const navigate = useNavigate()
    const [formData, setFormData] = useState({
        name: '',
        description: '',
        script_type: 'content_loader'
    })

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        try {
            let defaultContent = '// Rhai script\nfn main() {\n    print("Hello Block!");\n}\n'

            if (formData.script_type === 'transformer') {
                defaultContent = `// Transformer Script
// 'settings' is a Map of playback options
fn transform(settings) {
    // Example: settings.loop = true;
    return settings;
}`
            } else if (formData.script_type === 'content_loader') {
                defaultContent = `// Content Loader Script
// Return a valid ContentItem structure or a list of them
fn load_content() {
    return #{
        title: "My New Content",
        content_type: "remote_url", // or "local_file"
        content_path: "https://example.com/video.mp4",
        duration_minutes: 5,
    };
}`
            } else if (formData.script_type === 'server_context') {
                defaultContent = `// Server Context Script
// Runs on the server to inject real-world info into the DJ Prompt
// Available helpers: get_local_time(), http_get(url)

let time = get_local_time();
let context = "Current Time: " + time;

// You can fetch external data:
// let weather = http_get("https://wttr.in/?format=3");
// context += "\\nWeather: " + weather;

context; // The last expression is returned and appended to the DJ prompt
`
            } else if (formData.script_type === 'global') {
                defaultContent = `// Global Script
// Executed on Node during playback lifecycle events

fn on_load(settings) {
    // Called when content loads
    print("Global on_load");
}

fn on_unload(settings) {
    // Called when content unloads
    print("Global on_unload");
}
`
            }

            const newScript = await createScript({
                ...formData,
                script_content: defaultContent,
                parameters_schema: '{}',
                is_builtin: false
            })
            onClose()
            // Navigate to editor
            if (newScript && newScript.id) {
                navigate(`/scripts/${newScript.id}`)
            }
        } catch (e) {
            console.error(e)
            alert('Failed to create script')
        }
    }

    if (!isOpen) return null

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border-color)] w-full max-w-md overflow-hidden animate-fade-in shadow-2xl">
                <div className="p-4 border-b border-[var(--border-color)] flex justify-between items-center bg-[var(--bg-tertiary)]">
                    <h2 className="text-lg font-bold text-white">New Script</h2>
                    <button onClick={onClose} className="text-[var(--text-secondary)] hover:text-white transition-colors">
                        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-6 space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Name</label>
                        <input
                            type="text"
                            required
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:border-indigo-500 outline-none"
                            value={formData.name}
                            onChange={e => setFormData({ ...formData, name: e.target.value })}
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Description</label>
                        <textarea
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:border-indigo-500 outline-none h-20"
                            value={formData.description}
                            onChange={e => setFormData({ ...formData, description: e.target.value })}
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Type</label>
                        <select
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:border-indigo-500 outline-none"
                            value={formData.script_type}
                            onChange={e => setFormData({ ...formData, script_type: e.target.value })}
                        >
                            <option value="content_loader">Content Loader</option>
                            <option value="transformer">Transformer</option>
                            <option value="server_context">Server Context Source</option>
                            <option value="global">Global Script</option>
                        </select>
                    </div>

                    <div className="flex justify-end gap-3 pt-4">
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-4 py-2 rounded-lg text-sm font-medium text-[var(--text-secondary)] hover:text-white hover:bg-[var(--bg-primary)] transition-colors"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            className="btn-primary"
                        >
                            Create
                        </button>
                    </div>
                </form>
            </div>
        </div>
    )
}
