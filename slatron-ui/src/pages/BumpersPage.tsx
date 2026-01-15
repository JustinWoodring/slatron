import { useEffect, useState } from 'react'
import { useBumperStore, Bumper } from '../stores/bumperStore'
import { BumperEditor } from '../components/Bumpers/BumperEditor'
import { format } from 'date-fns'
import { AVAILABLE_BUMPER_BACKS } from '../constants/bumperAssets'

const BUMPER_TYPES = ['station_ident', 'transition', 'show_opener', 'lower_third', 'custom']

export default function BumpersPage() {
    const {
        bumpers,
        bumperBacks,
        fetchBumpers,
        fetchBumperBacks,
        createBumper,
        updateBumper,
        deleteBumper,
        renderBumper,
        renderAllBumpers,
        renderBumperBack,
        renderAllBumperBacks,
        downloadBumperBack,
        uploadBumperBack,
        deleteBumperBack
    } = useBumperStore()

    const [activeTab, setActiveTab] = useState<'bumpers' | 'backgrounds'>('bumpers')

    // Editor state
    const [selectedBumper, setSelectedBumper] = useState<Partial<Bumper> | null>(null)
    const [isEditing, setIsEditing] = useState(false)
    const [editorContent, setEditorContent] = useState('')
    const [editorForm, setEditorForm] = useState({
        name: '',
        bumper_type: 'station_ident',
        description: '',
        bumper_back_id: null as number | null
    })
    const [dirty, setDirty] = useState(false)
    const [renderingId, setRenderingId] = useState<number | null>(null)
    const [renderingBackId, setRenderingBackId] = useState<number | null>(null)
    const [renderingAll, setRenderingAll] = useState(false)
    const [filterType, setFilterType] = useState<string>('all')
    const [previewBumper, setPreviewBumper] = useState<Bumper | null>(null)

    // Backgrounds Modal State
    const [showBackgroundModal, setShowBackgroundModal] = useState(false)
    const [backgroundTab, setBackgroundTab] = useState<'library' | 'upload'>('library')
    const [uploadFile, setUploadFile] = useState<File | null>(null)
    const [uploadName, setUploadName] = useState('')
    const [processingBackground, setProcessingBackground] = useState<string | null>(null)

    useEffect(() => {
        fetchBumpers()
        fetchBumperBacks()
    }, [])

    const handleEdit = (bumper: Bumper) => {
        setSelectedBumper(bumper)
        setEditorContent(bumper.template_content || '')
        setEditorForm({
            name: bumper.name,
            bumper_type: bumper.bumper_type,
            description: bumper.description || '',
            bumper_back_id: bumper.bumper_back_id || null
        })
        setIsEditing(true)
        setDirty(false)
    }

    const handleNew = () => {
        const template = `<?xml version="1.0"?>
<mlt LC_NUMERIC="C" version="7.0.0" root="">
  <profile description="HD 1080p 30 fps" width="1920" height="1080"
          progressive="1" sample_aspect_num="1" sample_aspect_den="1"
          display_aspect_num="16" display_aspect_den="9"
          frame_rate_num="30" frame_rate_den="1" colorspace="709"/>
  
  <!-- Background Producer -->
  <producer id="background">
    <property name="resource">{{BUMPER_BACK_PATH}}</property>
    <property name="length">150</property> 
  </producer>

  <!-- Playlist -->
  <playlist id="main_playlist">
    <entry producer="background"/>
  </playlist>
  
  <!-- Add your text filters/overlays here using {{STATION_NAME}} -->
</mlt>`
        setSelectedBumper({ is_template: true })
        setEditorContent(template)
        setEditorForm({
            name: '',
            bumper_type: 'station_ident',
            description: '',
            bumper_back_id: null
        })
        setIsEditing(true)
        setDirty(true)
    }

    // Returns the saved bumper ID
    const saveBumperInternal = async (): Promise<number | null> => {
        if (!editorForm.name) {
            alert('Name is required')
            return null
        }

        try {
            const data = {
                name: editorForm.name,
                bumper_type: editorForm.bumper_type as any,
                description: editorForm.description,
                template_content: editorContent,
                is_template: true,
                is_builtin: selectedBumper?.is_builtin || false,
                bumper_back_id: editorForm.bumper_back_id || undefined
            }

            let savedBumper: Bumper;
            if (selectedBumper?.id) {
                savedBumper = await updateBumper(selectedBumper.id, data)
            } else {
                savedBumper = await createBumper(data)
            }

            // Update local state to reflect saved status
            setDirty(false)
            setSelectedBumper(savedBumper)
            fetchBumpers()
            return savedBumper.id
        } catch (e: any) {
            console.error(e)
            alert('Failed to save bumper: ' + e.message)
            return null
        }
    }

    const handleSave = async () => {
        const id = await saveBumperInternal()
        if (id) {
            setIsEditing(false)
            setSelectedBumper(null)
        }
    }

    const handleSaveAndRender = async (e: React.MouseEvent) => {
        e.preventDefault()
        e.stopPropagation()

        let id = selectedBumper?.id

        // If dirty or new, save first
        if (dirty || !id) {
            const savedId = await saveBumperInternal()
            if (!savedId) return // Save failed
            id = savedId
        }

        if (id) {
            await handleRender(id, e)
        }
    }

    const handleDelete = async (id: number) => {
        if (!window.confirm('Are you sure you want to delete this bumper?')) return
        try {
            await deleteBumper(id)
        } catch (e) {
            console.error(e)
            alert('Failed to delete bumper')
        }
    }

    const handleDownloadBack = async (asset: typeof AVAILABLE_BUMPER_BACKS[0]) => {
        setProcessingBackground(asset.url)
        try {
            await downloadBumperBack(asset.url, asset.name)
            alert(`Downloaded ${asset.name}!`)
            setShowBackgroundModal(false)
        } catch (e: any) {
            alert(`Failed to download: ${e.message || 'Unknown error'}`)
        } finally {
            setProcessingBackground(null)
        }
    }

    const handleUploadBack = async () => {
        if (!uploadFile) return
        setProcessingBackground('upload')
        try {
            await uploadBumperBack(uploadFile, uploadName || uploadFile.name)
            alert('Upload successful!')
            setShowBackgroundModal(false)
            setUploadFile(null)
            setUploadName('')
        } catch (e: any) {
            alert(`Failed to upload: ${e.message || 'Unknown error'}`)
        } finally {
            setProcessingBackground(null)
        }
    }

    const handleDeleteBack = async (id: number) => {
        if (!window.confirm('Delete this background? Bumpers using it will fail to render.')) return
        try {
            await deleteBumperBack(id)
        } catch (e: any) {
            if (e.response && e.response.status === 409) {
                alert('Cannot delete: This background is currently used by one or more bumpers.')
            } else {
                alert('Failed to delete background: ' + (e.message || 'Unknown error'))
            }
        }
    }

    const handleRender = async (id: number, e: React.MouseEvent) => {
        e.stopPropagation()
        try {
            setRenderingId(id)
            const res = await renderBumper(id)

            if (res.success) {
                // Update selectedBumper with new render path to show preview immediately
                if (selectedBumper && selectedBumper.id === id) {
                    setSelectedBumper({
                        ...selectedBumper,
                        rendered_path: res.rendered_path,
                        duration_ms: res.duration_ms
                    })
                }
            } else {
                alert('Render failed: ' + (res.error || 'Unknown error'))
            }
        } catch (e: any) {
            console.error(e)
            alert('Failed to render bumper: ' + (e.message || 'Network error'))
        } finally {
            setRenderingId(null)
        }
    }

    const handleRenderAll = async () => {
        if (!window.confirm('Render all bumpers? This may take a while.')) return
        try {
            setRenderingAll(true)
            const res = await renderAllBumpers()
            alert(`Render complete. Success: ${res.successful}, Failed: ${res.failed}`)
        } catch (e) {
            console.error(e)
            alert('Failed to render all bumpers')
        } finally {
            setRenderingAll(false)
        }
    }

    if (isEditing) {
        return (
            <div className="h-full flex flex-col p-6 gap-6">
                <div className="flex justify-between items-center">
                    <div className="flex items-center gap-4">
                        <button
                            onClick={() => {
                                if (dirty && !window.confirm('Discard changes?')) return
                                setIsEditing(false)
                            }}
                            className="text-[var(--text-secondary)] hover:text-white"
                        >
                            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                            </svg>
                        </button>
                        <h1 className="text-2xl font-bold bg-gradient-to-r from-pink-400 to-purple-400 bg-clip-text text-transparent">
                            {selectedBumper?.id ? `Edit ${editorForm.name}` : 'New Bumper'}
                        </h1>
                    </div>
                    <div className="flex items-center gap-3">
                        <button onClick={handleSave} className="btn-primary">
                            Save Bumper
                        </button>
                    </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1 min-h-0">
                    <div className="lg:col-span-2 h-full flex flex-col">
                        <BumperEditor
                            value={editorContent}
                            onChange={(val) => {
                                setEditorContent(val)
                                setDirty(true)
                            }}
                        />
                    </div>
                    <div className="flex flex-col gap-4">
                        <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl p-4 flex flex-col gap-4">
                            <h3 className="font-bold text-white flex items-center gap-2">Settings</h3>
                            <div>
                                <label className="text-xs font-medium text-[var(--text-secondary)] mb-1 block">Name</label>
                                <input
                                    type="text"
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-sm text-gray-200 focus:border-purple-500 outline-none transition-colors"
                                    value={editorForm.name}
                                    onChange={e => {
                                        setEditorForm({ ...editorForm, name: e.target.value })
                                        setDirty(true)
                                    }}
                                />
                            </div>
                            <div>
                                <label className="text-xs font-medium text-[var(--text-secondary)] mb-1 block">Type</label>
                                <select
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-sm text-gray-200 focus:border-purple-500 outline-none transition-colors"
                                    value={editorForm.bumper_type}
                                    onChange={e => {
                                        setEditorForm({ ...editorForm, bumper_type: e.target.value })
                                        setDirty(true)
                                    }}
                                >
                                    {BUMPER_TYPES.map(t => (
                                        <option key={t} value={t}>{t.replace('_', ' ').toUpperCase()}</option>
                                    ))}
                                </select>
                            </div>
                            <div>
                                <label className="text-xs font-medium text-[var(--text-secondary)] mb-1 block">Background Video</label>
                                <select
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-sm text-gray-200 focus:border-purple-500 outline-none transition-colors"
                                    value={editorForm.bumper_back_id || ''}
                                    onChange={e => {
                                        setEditorForm({ ...editorForm, bumper_back_id: e.target.value ? Number(e.target.value) : null })
                                        setDirty(true)
                                    }}
                                >
                                    <option value="">No Background (Only Colors/Variables)</option>
                                    {bumperBacks.map(b => (
                                        <option key={b.id} value={b.id}>{b.name} ({b.duration_ms ? (b.duration_ms / 1000).toFixed(1) + 's' : '?'})</option>
                                    ))}
                                </select>
                                <p className="text-[10px] text-[var(--text-secondary)] mt-1">
                                    Replaces <code>{'{{BUMPER_BACK_PATH}}'}</code> in the template.
                                </p>
                            </div>
                            <div>
                                <label className="text-xs font-medium text-[var(--text-secondary)] mb-1 block">Description</label>
                                <textarea
                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-sm text-gray-200 focus:border-purple-500 outline-none transition-colors h-24 resize-none"
                                    value={editorForm.description}
                                    onChange={e => {
                                        setEditorForm({ ...editorForm, description: e.target.value })
                                        setDirty(true)
                                    }}
                                />
                            </div>
                        </div>

                        {/* Rendering status / preview - same as before */}
                        {(selectedBumper?.id || isEditing) && (
                            <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl p-4 flex flex-col gap-4">
                                <h3 className="font-bold text-white flex items-center gap-2">Preview & Render</h3>
                                <button
                                    onClick={handleSaveAndRender}
                                    disabled={!!renderingId || (dirty && !editorForm.name)}
                                    className="w-full bg-purple-600 hover:bg-purple-700 text-white py-2 rounded-lg font-medium transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                                >
                                    {renderingId === selectedBumper?.id ? 'Rendering...' : 'Save & Render Now'}
                                </button>
                                {selectedBumper?.rendered_path && (
                                    <div className="mt-2 rounded-lg overflow-hidden border border-[var(--border-color)] bg-black aspect-video">
                                        <video
                                            src={`${import.meta.env.VITE_API_URL || ''}/${selectedBumper.rendered_path}?t=${Date.now()}`}
                                            controls
                                            className="w-full h-full object-contain"
                                        />
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        )
    }

    return (
        <div className="h-full flex flex-col p-6 gap-6">
            <div className="flex justify-between items-center">
                <div>
                    <h1 className="text-3xl font-bold bg-gradient-to-r from-pink-400 to-purple-400 bg-clip-text text-transparent">
                        Station Bumpers
                    </h1>
                    <p className="text-[var(--text-secondary)] mt-1">Manage idents, transitions, and backgrounds</p>
                </div>
            </div>

            {/* Tabs */}
            <div className="flex gap-4 border-b border-[var(--border-color)]">
                <button
                    className={`pb-3 px-2 text-sm font-medium transition-colors ${activeTab === 'bumpers' ? 'text-purple-400 border-b-2 border-purple-400' : 'text-[var(--text-secondary)] hover:text-white'}`}
                    onClick={() => setActiveTab('bumpers')}
                >
                    Bumpers
                </button>
                <button
                    className={`pb-3 px-2 text-sm font-medium transition-colors ${activeTab === 'backgrounds' ? 'text-purple-400 border-b-2 border-purple-400' : 'text-[var(--text-secondary)] hover:text-white'}`}
                    onClick={() => setActiveTab('backgrounds')}
                >
                    Background Videos
                </button>
            </div>

            {activeTab === 'bumpers' ? (
                // BUMPERS TAB
                <div className="flex-1 flex flex-col gap-4">
                    <div className="flex justify-end gap-3">
                        <select
                            className="bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-secondary)] text-sm rounded-lg px-3 py-2"
                            value={filterType}
                            onChange={(e) => setFilterType(e.target.value)}
                        >
                            <option value="all">All Types</option>
                            {BUMPER_TYPES.map(t => (
                                <option key={t} value={t}>{t.replace('_', ' ').toUpperCase()}</option>
                            ))}
                        </select>
                        <button
                            onClick={handleRenderAll}
                            disabled={renderingAll}
                            className="btn-secondary"
                        >
                            Render All
                        </button>
                        <button onClick={handleNew} className="btn-primary">
                            New Bumper
                        </button>
                    </div>

                    <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl overflow-hidden flex-1">
                        <div className="overflow-x-auto">
                            <table className="w-full text-left">
                                <thead className="bg-[var(--bg-tertiary)] text-xs uppercase text-[var(--text-secondary)]">
                                    <tr>
                                        <th className="p-4">Name</th>
                                        <th className="p-4">Type</th>
                                        <th className="p-4">Description</th>
                                        <th className="p-4">Status</th>
                                        <th className="p-4 text-right">Actions</th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-[var(--border-color)]">
                                    {bumpers.filter(b => filterType === 'all' || b.bumper_type === filterType).map(bumper => (
                                        <tr key={bumper.id} className="hover:bg-white/5 group">
                                            <td className="p-4">
                                                <div className="font-medium text-white">{bumper.name}</div>
                                                {bumper.is_builtin && <span className="text-[10px] bg-blue-500/10 text-blue-400 px-1 rounded">BUILT-IN</span>}
                                            </td>
                                            <td className="p-4"><span className="text-xs font-mono bg-[var(--bg-primary)] px-2 py-1 rounded">{bumper.bumper_type}</span></td>
                                            <td className="p-4 text-sm text-[var(--text-secondary)] truncate max-w-xs">{bumper.description}</td>
                                            <td className="p-4">
                                                {renderingId === bumper.id ? <span className="text-purple-400">Rendering...</span> :
                                                    bumper.rendered_path ? <span className="text-emerald-400">Ready</span> :
                                                        <span className="text-amber-400">Needs Render</span>}
                                            </td>
                                            <td className="p-4 text-right">
                                                <div className="flex justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                                    <button
                                                        onClick={() => setPreviewBumper(bumper)}
                                                        className="p-2 hover:bg-emerald-500/10 text-emerald-400 rounded-lg transition-colors"
                                                        title="Preview"
                                                    >
                                                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                                        </svg>
                                                    </button>
                                                    <button
                                                        onClick={(e) => handleRender(bumper.id, e)}
                                                        className="p-2 hover:bg-purple-500/10 text-purple-400 rounded-lg transition-colors"
                                                        title="Render"
                                                    >
                                                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                                                        </svg>
                                                    </button>
                                                    <button
                                                        onClick={() => handleEdit(bumper)}
                                                        className="p-2 hover:bg-blue-500/10 text-blue-400 rounded-lg transition-colors"
                                                        title="Edit"
                                                    >
                                                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                        </svg>
                                                    </button>
                                                    <button
                                                        onClick={() => handleDelete(bumper.id)}
                                                        className="p-2 hover:bg-red-500/10 text-red-400 rounded-lg transition-colors"
                                                        title="Delete"
                                                    >
                                                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                        </svg>
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            ) : (
                // BACKGROUNDS TAB
                <div className="flex-1 flex flex-col gap-4">
                    <div className="flex justify-between items-center">
                        <p className="text-[var(--text-secondary)] text-sm">
                            Library of video loops used as backgrounds for bumpers.
                        </p>
                        <div className="flex gap-3">
                            <button
                                onClick={async () => {
                                    if (!window.confirm('Render all MLT backgrounds?')) return
                                    try {
                                        setRenderingAll(true)
                                        const res = await renderAllBumperBacks()
                                        alert(`Render complete. Success: ${res.successful}, Failed: ${res.failed}`)
                                    } catch (e) {
                                        console.error(e)
                                        alert('Failed to render backgrounds')
                                    } finally {
                                        setRenderingAll(false)
                                    }
                                }}
                                disabled={renderingAll}
                                className="btn-secondary"
                            >
                                Render All MLT
                            </button>
                            <button onClick={() => setShowBackgroundModal(true)} className="btn-primary">
                                Add Background
                            </button>
                        </div>
                    </div>

                    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                        {bumperBacks.map(back => {
                            const isMlt = back.file_path.endsWith('.mlt')
                            return (
                                <div key={back.id} className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl overflow-hidden group">
                                    <div className="aspect-video bg-black relative">
                                        <video
                                            src={`${import.meta.env.VITE_API_URL || ''}/${back.file_path}`}
                                            className="w-full h-full object-cover"
                                            onMouseOver={e => e.currentTarget.play()}
                                            onMouseOut={e => { e.currentTarget.pause(); e.currentTarget.currentTime = 0; }}
                                            muted
                                            loop
                                        />
                                        <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                                            {isMlt && (
                                                <button
                                                    onClick={async () => {
                                                        try {
                                                            setRenderingBackId(back.id)
                                                            await renderBumperBack(back.id)
                                                        } catch (e) {
                                                            console.error(e)
                                                            alert('Failed to render background')
                                                        } finally {
                                                            setRenderingBackId(null)
                                                        }
                                                    }}
                                                    className="p-2 bg-purple-500/80 hover:bg-purple-500 text-white rounded-full"
                                                    title="Render MLT"
                                                >
                                                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                                                    </svg>
                                                </button>
                                            )}
                                            <button
                                                onClick={() => handleDeleteBack(back.id)}
                                                className="p-2 bg-red-500/80 hover:bg-red-500 text-white rounded-full"
                                            >
                                                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                                            </button>
                                        </div>
                                        {/* Status Indicators */}
                                        {renderingBackId === back.id && (
                                            <div className="absolute inset-0 bg-black/80 flex flex-col items-center justify-center text-purple-400 text-xs font-medium">
                                                <svg className="animate-spin h-6 w-6 mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                </svg>
                                                Rendering...
                                            </div>
                                        )}
                                        {isMlt && !renderingBackId && (
                                            <div className="absolute top-2 right-2 px-1.5 py-0.5 rounded bg-black/60 text-[10px] font-mono border border-white/20 text-white">
                                                MLT
                                            </div>
                                        )}
                                    </div>
                                    <div className="p-3">
                                        <div className="font-medium text-white truncate" title={back.name}>{back.name}</div>
                                        <div className="text-xs text-[var(--text-secondary)] mt-1 flex justify-between">
                                            <span>{back.duration_ms ? (back.duration_ms / 1000).toFixed(1) + 's' : '?'}</span>
                                            <span>{format(new Date(back.created_at), 'MMM d')}</span>
                                        </div>
                                    </div>
                                </div>
                            )
                        })}
                    </div>
                </div>
            )}

            {/* ADD BACKGROUND MODAL */}
            {showBackgroundModal && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm" onClick={() => setShowBackgroundModal(false)}>
                    <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl overflow-hidden max-w-2xl w-full flex flex-col shadow-2xl" onClick={e => e.stopPropagation()}>
                        <div className="p-4 border-b border-[var(--border-color)] bg-[var(--bg-tertiary)] flex justify-between">
                            <h3 className="font-bold text-white">Add Background</h3>
                            <button onClick={() => setShowBackgroundModal(false)} className="text-[var(--text-secondary)] hover:text-white">✕</button>
                        </div>

                        <div className="p-4 flex gap-4 border-b border-[var(--border-color)]">
                            <button
                                className={`pb-2 text-sm font-medium ${backgroundTab === 'library' ? 'text-purple-400 border-b-2 border-purple-400' : 'text-[var(--text-secondary)]'}`}
                                onClick={() => setBackgroundTab('library')}
                            >
                                From Library
                            </button>
                            <button
                                className={`pb-2 text-sm font-medium ${backgroundTab === 'upload' ? 'text-purple-400 border-b-2 border-purple-400' : 'text-[var(--text-secondary)]'}`}
                                onClick={() => setBackgroundTab('upload')}
                            >
                                Upload File
                            </button>
                        </div>

                        <div className="p-4 max-h-[60vh] overflow-y-auto">
                            {backgroundTab === 'library' ? (
                                <div className="grid grid-cols-2 gap-4">
                                    {AVAILABLE_BUMPER_BACKS.map((asset, i) => (
                                        <div key={i} className="border border-[var(--border-color)] rounded-lg p-3 hover:bg-[var(--bg-tertiary)] transition-colors">
                                            <div className="font-medium text-white">{asset.name}</div>
                                            <p className="text-xs text-[var(--text-secondary)] mb-2">{asset.description}</p>
                                            <button
                                                onClick={() => handleDownloadBack(asset)}
                                                disabled={!!processingBackground}
                                                className="w-full btn-secondary text-xs py-1"
                                            >
                                                {processingBackground === asset.url ? 'Downloading...' : 'Download'}
                                            </button>
                                        </div>
                                    ))}
                                </div>
                            ) : (
                                <div className="flex flex-col gap-4">
                                    <div className="border-2 border-dashed border-[var(--border-color)] rounded-xl p-8 text-center hover:border-purple-500/50 transition-colors">
                                        <input
                                            type="file"
                                            accept="video/*"
                                            onChange={e => {
                                                if (e.target.files?.[0]) {
                                                    setUploadFile(e.target.files[0])
                                                    setUploadName(e.target.files[0].name.split('.')[0])
                                                }
                                            }}
                                            className="hidden"
                                            id="file-upload"
                                        />
                                        <label htmlFor="file-upload" className="cursor-pointer block">
                                            {uploadFile ? (
                                                <div className="text-emerald-400 font-medium">{uploadFile.name}</div>
                                            ) : (
                                                <>
                                                    <div className="text-purple-400 mb-2">Click to select video</div>
                                                    <div className="text-xs text-[var(--text-secondary)]">MP4, WEBM, MOV supported</div>
                                                </>
                                            )}
                                        </label>
                                    </div>
                                    {uploadFile && (
                                        <>
                                            <div>
                                                <label className="text-xs font-medium text-[var(--text-secondary)] block mb-1">Name</label>
                                                <input
                                                    type="text"
                                                    value={uploadName}
                                                    onChange={e => setUploadName(e.target.value)}
                                                    className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white"
                                                />
                                            </div>
                                            <button
                                                onClick={handleUploadBack}
                                                disabled={!!processingBackground}
                                                className="btn-primary w-full"
                                            >
                                                {processingBackground ? 'Uploading...' : 'Upload & Save'}
                                            </button>
                                        </>
                                    )}
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            )}
            {/* BUMPER PREVIEW MODAL */}
            {previewBumper && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/90 backdrop-blur-sm transition-opacity" onClick={() => setPreviewBumper(null)}>
                    <div className="relative bg-[#0f0f12] border border-white/10 rounded-2xl overflow-hidden max-w-5xl w-full flex flex-col shadow-2xl ring-1 ring-white/10" onClick={e => e.stopPropagation()}>
                        <div className="absolute top-0 left-0 right-0 z-10 p-4 flex justify-between items-start bg-gradient-to-b from-black/80 to-transparent pointer-events-none">
                            <div className="pointer-events-auto">
                                <h3 className="font-bold text-white text-lg drop-shadow-md">{previewBumper.name}</h3>
                                <div className="text-xs text-white/70 font-mono mt-0.5 uppercase tracking-wider">{previewBumper.bumper_type.replace('_', ' ')}</div>
                            </div>
                            <button
                                onClick={() => setPreviewBumper(null)}
                                className="pointer-events-auto bg-black/50 hover:bg-white/20 text-white rounded-full p-2 backdrop-blur-sm transition-all"
                            >
                                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>

                        <div className="aspect-video bg-black flex items-center justify-center relative group">
                            {previewBumper.rendered_path ? (
                                <video
                                    src={`${import.meta.env.VITE_API_URL || ''}/${previewBumper.rendered_path}`}
                                    controls
                                    autoPlay
                                    className="w-full h-full object-contain focus:outline-none"
                                />
                            ) : (
                                <div className="flex flex-col items-center gap-4 p-8 text-center max-w-sm">
                                    <div className="w-20 h-20 rounded-full bg-white/5 flex items-center justify-center mb-2">
                                        <svg className="w-10 h-10 text-white/20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                                        </svg>
                                    </div>
                                    <div className="space-y-1">
                                        <h4 className="text-white font-medium">No Render Available</h4>
                                        <p className="text-sm text-[var(--text-secondary)]">
                                            This bumper hasn't been rendered yet. You need to render it before you can preview the animation.
                                        </p>
                                    </div>
                                    <button
                                        onClick={() => {
                                            if (previewBumper.id) {
                                                setPreviewBumper(null)
                                                handleRender(previewBumper.id, { stopPropagation: () => { } } as React.MouseEvent)
                                            }
                                        }}
                                        className="btn-primary mt-2 flex items-center gap-2"
                                    >
                                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                                        </svg>
                                        Render Now
                                    </button>
                                </div>
                            )}
                        </div>

                        {previewBumper.rendered_path && (
                            <div className="bg-[#0f0f12] p-4 text-xs text-[var(--text-secondary)] border-t border-white/5 flex justify-between items-center">
                                <span>Duration: {previewBumper.duration_ms ? (previewBumper.duration_ms / 1000).toFixed(1) + 's' : 'Unknown'}</span>
                                <span className="font-mono opacity-50">{previewBumper.rendered_path}</span>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    )
}
