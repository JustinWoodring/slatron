import { useEffect, useState, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useSpotReelStore, SpotReelItem } from '../stores/spotReelStore'
import { useAuthStore } from '../stores/authStore'

const ITEM_TYPE_ICONS: Record<string, { icon: JSX.Element; color: string; label: string }> = {
  image: {
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
    ),
    color: 'text-green-400 bg-green-500/20 border-green-500/30',
    label: 'Image',
  },
  video: {
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
      </svg>
    ),
    color: 'text-blue-400 bg-blue-500/20 border-blue-500/30',
    label: 'Video',
  },
  web: {
    icon: (
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
      </svg>
    ),
    color: 'text-orange-400 bg-orange-500/20 border-orange-500/30',
    label: 'Web Page',
  },
}

export default function SpotReelEditorPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { user } = useAuthStore()
  const isEditor = user?.role === 'admin' || user?.role === 'editor'

  const {
    currentReel,
    loading,
    fetchSpotReel,
    updateSpotReel,
    addItem,
    updateItem,
    deleteItem,
    reorderItems,
  } = useSpotReelStore()

  // Reel metadata editing
  const [editTitle, setEditTitle] = useState('')
  const [editDescription, setEditDescription] = useState('')
  const [isDirty, setIsDirty] = useState(false)
  const [saving, setSaving] = useState(false)

  // Add item form
  const [showAddItem, setShowAddItem] = useState(false)
  const [newItemType, setNewItemType] = useState<'image' | 'video' | 'web'>('image')
  const [newItemPath, setNewItemPath] = useState('')
  const [newItemTitle, setNewItemTitle] = useState('')
  const [newItemDuration, setNewItemDuration] = useState(10)
  const [addingItem, setAddingItem] = useState(false)

  // Inline edit state
  const [editingItemId, setEditingItemId] = useState<number | null>(null)
  const [editItemDuration, setEditItemDuration] = useState(10)
  const [editItemTitle, setEditItemTitle] = useState('')
  const [editItemPath, setEditItemPath] = useState('')

  // Drag state
  const [dragIndex, setDragIndex] = useState<number | null>(null)
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null)

  useEffect(() => {
    if (id) {
      fetchSpotReel(parseInt(id))
    }
  }, [id])

  useEffect(() => {
    if (currentReel) {
      setEditTitle(currentReel.title)
      setEditDescription(currentReel.description || '')
    }
  }, [currentReel])

  // Track dirty state for metadata
  useEffect(() => {
    if (!currentReel) return
    const titleChanged = editTitle !== currentReel.title
    const descChanged = editDescription !== (currentReel.description || '')
    setIsDirty(titleChanged || descChanged)
  }, [editTitle, editDescription, currentReel])

  const handleSaveMetadata = async () => {
    if (!currentReel || !isDirty) return
    setSaving(true)
    try {
      await updateSpotReel(currentReel.id, {
        title: editTitle.trim(),
        description: editDescription.trim() || undefined,
      })
      setIsDirty(false)
    } catch (err) {
      console.error('Failed to update spot reel:', err)
    } finally {
      setSaving(false)
    }
  }

  const handleAddItem = async () => {
    if (!currentReel || !newItemPath.trim()) return
    setAddingItem(true)
    try {
      await addItem(currentReel.id, {
        item_type: newItemType,
        item_path: newItemPath.trim(),
        display_duration_secs: newItemDuration,
        title: newItemTitle.trim() || undefined,
      })
      setNewItemPath('')
      setNewItemTitle('')
      setNewItemDuration(10)
      setShowAddItem(false)
    } catch (err) {
      console.error('Failed to add item:', err)
    } finally {
      setAddingItem(false)
    }
  }

  const handleDeleteItem = async (item: SpotReelItem) => {
    if (!currentReel) return
    if (!confirm(`Remove "${item.title || item.item_path}" from this reel?`)) return
    try {
      await deleteItem(currentReel.id, item.id)
    } catch (err) {
      console.error('Failed to delete item:', err)
    }
  }

  const handleStartEditItem = (item: SpotReelItem) => {
    setEditingItemId(item.id)
    setEditItemDuration(item.display_duration_secs)
    setEditItemTitle(item.title || '')
    setEditItemPath(item.item_path)
  }

  const handleSaveItemEdit = async () => {
    if (!currentReel || editingItemId === null) return
    try {
      await updateItem(currentReel.id, editingItemId, {
        display_duration_secs: editItemDuration,
        title: editItemTitle.trim() || undefined,
        item_path: editItemPath.trim(),
      } as Partial<SpotReelItem>)
      setEditingItemId(null)
    } catch (err) {
      console.error('Failed to update item:', err)
    }
  }

  const handleCancelItemEdit = () => {
    setEditingItemId(null)
  }

  // Drag-and-drop reorder
  const handleDragStart = useCallback((index: number) => {
    setDragIndex(index)
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent, index: number) => {
    e.preventDefault()
    setDragOverIndex(index)
  }, [])

  const handleDrop = useCallback(async (targetIndex: number) => {
    if (dragIndex === null || dragIndex === targetIndex || !currentReel) {
      setDragIndex(null)
      setDragOverIndex(null)
      return
    }

    const items = [...currentReel.items]
    const [moved] = items.splice(dragIndex, 1)
    items.splice(targetIndex, 0, moved)

    // Build reorder payload
    const reorderPayload = items.map((item, idx) => ({ id: item.id, position: idx }))

    setDragIndex(null)
    setDragOverIndex(null)

    try {
      await reorderItems(currentReel.id, reorderPayload)
    } catch (err) {
      console.error('Failed to reorder items:', err)
    }
  }, [dragIndex, currentReel, reorderItems])

  const handleDragEnd = useCallback(() => {
    setDragIndex(null)
    setDragOverIndex(null)
  }, [])

  const formatDuration = (secs: number) => {
    if (secs < 60) return `${secs}s`
    const m = Math.floor(secs / 60)
    const s = secs % 60
    return s > 0 ? `${m}m ${s}s` : `${m}m`
  }

  if (loading && !currentReel) {
    return (
      <div className="flex items-center justify-center py-20 animate-fade-in">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-500" />
      </div>
    )
  }

  if (!currentReel) {
    return (
      <div className="animate-fade-in text-center py-20">
        <p className="text-[var(--text-secondary)]">Spot reel not found</p>
        <button onClick={() => navigate('/spot-reels')} className="btn-secondary mt-4">
          Back to Spot Reels
        </button>
      </div>
    )
  }

  const totalDuration = currentReel.items.reduce((sum, i) => sum + i.display_duration_secs, 0)

  return (
    <div className="animate-fade-in">
      {/* Header */}
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={() => navigate('/spot-reels')}
          className="p-2 hover:bg-[var(--bg-tertiary)] rounded-lg text-[var(--text-secondary)] hover:text-white transition-colors"
        >
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <div className="flex-1">
          <h1 className="text-2xl font-bold text-white">{currentReel.title}</h1>
          <p className="text-sm text-[var(--text-secondary)]">
            {currentReel.items.length} item{currentReel.items.length !== 1 ? 's' : ''} &middot; {formatDuration(totalDuration)} total
            {currentReel.content_item_id && (
              <span className="ml-2 text-xs text-purple-400">(Content #{currentReel.content_item_id})</span>
            )}
          </p>
        </div>
        {isEditor && isDirty && (
          <button
            onClick={handleSaveMetadata}
            disabled={saving}
            className="btn-primary disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left: Item List (2/3) */}
        <div className="lg:col-span-2 space-y-4">
          {/* Add Item Bar */}
          {isEditor && (
            <div className="glass-panel border border-[var(--border-color)] rounded-xl p-4">
              {!showAddItem ? (
                <button
                  onClick={() => setShowAddItem(true)}
                  className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-white transition-colors"
                >
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                  </svg>
                  Add Item
                </button>
              ) : (
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <h3 className="text-sm font-semibold text-white">Add New Item</h3>
                    <button
                      onClick={() => setShowAddItem(false)}
                      className="p-1 hover:bg-[var(--bg-tertiary)] rounded text-[var(--text-secondary)] hover:text-white transition-colors"
                    >
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>

                  {/* Type selector */}
                  <div className="flex gap-2">
                    {(['image', 'video', 'web'] as const).map((type) => {
                      const info = ITEM_TYPE_ICONS[type]
                      return (
                        <button
                          key={type}
                          onClick={() => setNewItemType(type)}
                          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors ${
                            newItemType === type
                              ? info.color
                              : 'text-[var(--text-secondary)] border-[var(--border-color)] hover:bg-[var(--bg-tertiary)]'
                          }`}
                        >
                          {info.icon}
                          {info.label}
                        </button>
                      )
                    })}
                  </div>

                  {/* Fields */}
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    <div className="sm:col-span-2">
                      <label className="block text-xs text-[var(--text-secondary)] mb-1">
                        {newItemType === 'web' ? 'URL' : 'Path / URL'}
                      </label>
                      <input
                        type="text"
                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                        placeholder={newItemType === 'web' ? 'https://example.com' : newItemType === 'image' ? '/path/to/image.png or https://...' : '/path/to/video.mp4 or https://...'}
                        value={newItemPath}
                        onChange={(e) => setNewItemPath(e.target.value)}
                      />
                    </div>
                    <div>
                      <label className="block text-xs text-[var(--text-secondary)] mb-1">Title (optional)</label>
                      <input
                        type="text"
                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                        placeholder="Item title"
                        value={newItemTitle}
                        onChange={(e) => setNewItemTitle(e.target.value)}
                      />
                    </div>
                    <div>
                      <label className="block text-xs text-[var(--text-secondary)] mb-1">Display Duration (seconds)</label>
                      <input
                        type="number"
                        min={1}
                        className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                        value={newItemDuration}
                        onChange={(e) => setNewItemDuration(parseInt(e.target.value) || 10)}
                      />
                    </div>
                  </div>

                  <div className="flex justify-end gap-2">
                    <button
                      onClick={() => setShowAddItem(false)}
                      className="px-3 py-1.5 rounded-lg text-sm text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] transition-colors"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleAddItem}
                      disabled={!newItemPath.trim() || addingItem}
                      className="btn-primary text-sm disabled:opacity-50"
                    >
                      {addingItem ? 'Adding...' : 'Add Item'}
                    </button>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Items List */}
          {currentReel.items.length === 0 ? (
            <div className="glass-panel border border-[var(--border-color)] rounded-xl p-12 text-center">
              <svg className="w-12 h-12 mx-auto text-[var(--text-secondary)] opacity-30 mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              <p className="text-[var(--text-secondary)]">No items in this reel yet</p>
              <p className="text-[var(--text-secondary)] text-xs mt-1 opacity-60">
                Add images, videos, or web pages to create your carousel
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {currentReel.items.map((item, index) => {
                const typeInfo = ITEM_TYPE_ICONS[item.item_type] || ITEM_TYPE_ICONS.image
                const isEditing = editingItemId === item.id

                return (
                  <div
                    key={item.id}
                    draggable={isEditor && !isEditing}
                    onDragStart={() => handleDragStart(index)}
                    onDragOver={(e) => handleDragOver(e, index)}
                    onDrop={() => handleDrop(index)}
                    onDragEnd={handleDragEnd}
                    className={`glass-panel border rounded-xl p-4 transition-all duration-200 group ${
                      dragOverIndex === index && dragIndex !== index
                        ? 'border-indigo-500 bg-indigo-500/10'
                        : dragIndex === index
                        ? 'border-[var(--border-color)] opacity-50'
                        : 'border-[var(--border-color)] hover:border-[var(--border-color)]'
                    }`}
                  >
                    {isEditing ? (
                      /* Inline Edit Mode */
                      <div className="space-y-3">
                        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                          <div className="sm:col-span-2">
                            <label className="block text-xs text-[var(--text-secondary)] mb-1">Path / URL</label>
                            <input
                              type="text"
                              className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:border-indigo-500"
                              value={editItemPath}
                              onChange={(e) => setEditItemPath(e.target.value)}
                            />
                          </div>
                          <div>
                            <label className="block text-xs text-[var(--text-secondary)] mb-1">Duration (s)</label>
                            <input
                              type="number"
                              min={1}
                              className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:border-indigo-500"
                              value={editItemDuration}
                              onChange={(e) => setEditItemDuration(parseInt(e.target.value) || 10)}
                            />
                          </div>
                        </div>
                        <div>
                          <label className="block text-xs text-[var(--text-secondary)] mb-1">Title (optional)</label>
                          <input
                            type="text"
                            className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:border-indigo-500"
                            value={editItemTitle}
                            onChange={(e) => setEditItemTitle(e.target.value)}
                            placeholder="Item title"
                          />
                        </div>
                        <div className="flex justify-end gap-2">
                          <button
                            onClick={handleCancelItemEdit}
                            className="px-3 py-1 rounded-lg text-xs text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] transition-colors"
                          >
                            Cancel
                          </button>
                          <button
                            onClick={handleSaveItemEdit}
                            className="btn-primary text-xs"
                          >
                            Save
                          </button>
                        </div>
                      </div>
                    ) : (
                      /* Display Mode */
                      <div className="flex items-center gap-3">
                        {/* Drag Handle */}
                        {isEditor && (
                          <div className="cursor-grab active:cursor-grabbing text-[var(--text-secondary)] hover:text-white transition-colors opacity-0 group-hover:opacity-100">
                            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                            </svg>
                          </div>
                        )}

                        {/* Position Number */}
                        <span className="text-xs font-mono text-[var(--text-secondary)] w-5 text-center">
                          {index + 1}
                        </span>

                        {/* Type Badge */}
                        <div className={`flex items-center justify-center w-8 h-8 rounded-lg border ${typeInfo.color}`}>
                          {typeInfo.icon}
                        </div>

                        {/* Info */}
                        <div className="flex-1 min-w-0">
                          <p className="text-sm text-white font-medium truncate">
                            {item.title || item.item_path}
                          </p>
                          {item.title && (
                            <p className="text-xs text-[var(--text-secondary)] truncate">
                              {item.item_path}
                            </p>
                          )}
                        </div>

                        {/* Duration */}
                        <span className="text-xs text-[var(--text-secondary)] flex items-center gap-1 flex-shrink-0">
                          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                          </svg>
                          {formatDuration(item.display_duration_secs)}
                        </span>

                        {/* Actions */}
                        {isEditor && (
                          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
                            <button
                              onClick={() => handleStartEditItem(item)}
                              className="p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-white transition-colors"
                              title="Edit item"
                            >
                              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                              </svg>
                            </button>
                            <button
                              onClick={() => handleDeleteItem(item)}
                              className="p-1.5 rounded-lg hover:bg-red-500/20 text-[var(--text-secondary)] hover:text-red-400 transition-colors"
                              title="Remove item"
                            >
                              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                              </svg>
                            </button>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          )}
        </div>

        {/* Right: Metadata Sidebar (1/3) */}
        <div className="space-y-4">
          {/* Reel Info Panel */}
          <div className="glass-panel border border-[var(--border-color)] rounded-xl p-4 space-y-4">
            <h3 className="text-sm font-semibold text-white">Reel Settings</h3>

            <div>
              <label className="block text-xs text-[var(--text-secondary)] mb-1">Title</label>
              <input
                type="text"
                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                value={editTitle}
                onChange={(e) => setEditTitle(e.target.value)}
                disabled={!isEditor}
              />
            </div>

            <div>
              <label className="block text-xs text-[var(--text-secondary)] mb-1">Description</label>
              <textarea
                className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500 resize-none"
                rows={3}
                value={editDescription}
                onChange={(e) => setEditDescription(e.target.value)}
                disabled={!isEditor}
                placeholder="Optional description..."
              />
            </div>

            {isEditor && isDirty && (
              <button
                onClick={handleSaveMetadata}
                disabled={saving}
                className="btn-primary w-full text-sm disabled:opacity-50"
              >
                {saving ? 'Saving...' : 'Save Changes'}
              </button>
            )}
          </div>

          {/* Stats Panel */}
          <div className="glass-panel border border-[var(--border-color)] rounded-xl p-4 space-y-3">
            <h3 className="text-sm font-semibold text-white">Statistics</h3>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs text-[var(--text-secondary)]">Total Items</span>
                <span className="text-sm text-white font-medium">{currentReel.items.length}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs text-[var(--text-secondary)]">Loop Duration</span>
                <span className="text-sm text-white font-medium">{formatDuration(totalDuration)}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs text-[var(--text-secondary)]">Content Item ID</span>
                <span className="text-sm text-white font-medium">
                  {currentReel.content_item_id ? `#${currentReel.content_item_id}` : 'N/A'}
                </span>
              </div>
            </div>

            {/* Item type breakdown */}
            {currentReel.items.length > 0 && (
              <div className="pt-3 border-t border-[var(--border-color)] space-y-1.5">
                {(['image', 'video', 'web'] as const).map((type) => {
                  const count = currentReel.items.filter((i) => i.item_type === type).length
                  if (count === 0) return null
                  const info = ITEM_TYPE_ICONS[type]
                  return (
                    <div key={type} className="flex items-center justify-between">
                      <span className={`text-xs flex items-center gap-1.5 ${info.color.split(' ')[0]}`}>
                        {info.icon}
                        {info.label}
                      </span>
                      <span className="text-xs text-[var(--text-secondary)]">{count}</span>
                    </div>
                  )
                })}
              </div>
            )}
          </div>

          {/* Help Panel */}
          <div className="glass-panel border border-[var(--border-color)] rounded-xl p-4 space-y-2">
            <h3 className="text-sm font-semibold text-white">How It Works</h3>
            <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
              A Spot Reel loops through its items in order. Each item displays for its configured duration, then the reel advances to the next item. When the last item finishes, it loops back to the first.
            </p>
            <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
              Assign this reel to a schedule block via its auto-created content item. The reel will loop until the block ends.
            </p>
            {isEditor && (
              <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                Drag items to reorder. Transitions are instant (hard cut).
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
