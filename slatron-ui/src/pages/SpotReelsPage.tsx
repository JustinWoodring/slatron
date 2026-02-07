import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useSpotReelStore, SpotReelListEntry } from '../stores/spotReelStore'
import { useAuthStore } from '../stores/authStore'

export default function SpotReelsPage() {
  const { reels, loading, fetchSpotReels, createSpotReel, deleteSpotReel } = useSpotReelStore()
  const { user } = useAuthStore()
  const isEditor = user?.role === 'admin' || user?.role === 'editor'
  const navigate = useNavigate()

  const [showCreate, setShowCreate] = useState(false)
  const [newTitle, setNewTitle] = useState('')
  const [newDescription, setNewDescription] = useState('')
  const [creating, setCreating] = useState(false)

  useEffect(() => {
    fetchSpotReels()
  }, [])

  const handleCreate = async () => {
    if (!newTitle.trim()) return
    setCreating(true)
    try {
      const reel = await createSpotReel(newTitle.trim(), newDescription.trim() || undefined)
      setShowCreate(false)
      setNewTitle('')
      setNewDescription('')
      navigate(`/spot-reels/${reel.id}`)
    } catch (err) {
      console.error('Failed to create spot reel:', err)
    } finally {
      setCreating(false)
    }
  }

  const handleDelete = async (e: React.MouseEvent, reel: SpotReelListEntry) => {
    e.stopPropagation()
    if (!confirm(`Delete "${reel.title}"? This will also remove the associated content item.`)) return
    try {
      await deleteSpotReel(reel.id)
    } catch (err) {
      console.error('Failed to delete spot reel:', err)
    }
  }

  const formatDuration = (secs: number) => {
    if (secs < 60) return `${secs}s`
    const m = Math.floor(secs / 60)
    const s = secs % 60
    return s > 0 ? `${m}m ${s}s` : `${m}m`
  }

  return (
    <div className="animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold bg-gradient-to-r from-white to-gray-400 bg-clip-text text-transparent">
            Spot Reels
          </h1>
          <p className="text-[var(--text-secondary)] mt-1">
            Create carousel playlists of images, videos, and web pages
          </p>
        </div>
        {isEditor && (
          <button
            onClick={() => setShowCreate(true)}
            className="btn-primary flex items-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
            New Spot Reel
          </button>
        )}
      </div>

      {/* Loading */}
      {loading && reels.length === 0 && (
        <div className="flex items-center justify-center py-20">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-500" />
        </div>
      )}

      {/* Empty State */}
      {!loading && reels.length === 0 && (
        <div className="text-center py-20">
          <svg className="w-16 h-16 mx-auto text-[var(--text-secondary)] opacity-30 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p className="text-[var(--text-secondary)] text-lg">No spot reels yet</p>
          <p className="text-[var(--text-secondary)] text-sm mt-1 opacity-60">
            Create one to bundle images, videos, and web pages into a looping carousel
          </p>
        </div>
      )}

      {/* Card Grid */}
      {reels.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {reels.map((reel) => (
            <div
              key={reel.id}
              onClick={() => navigate(`/spot-reels/${reel.id}`)}
              className="glass-panel border border-[var(--border-color)] rounded-xl p-5 cursor-pointer hover:border-indigo-500/50 hover:bg-[var(--bg-tertiary)]/50 transition-all duration-200 group relative"
            >
              {/* Icon */}
              <div className="w-10 h-10 rounded-lg bg-purple-500/20 flex items-center justify-center mb-3 border border-purple-500/30">
                <svg className="w-5 h-5 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
              </div>

              {/* Title */}
              <h3 className="text-white font-semibold text-sm truncate">{reel.title}</h3>

              {/* Description */}
              {reel.description && (
                <p className="text-[var(--text-secondary)] text-xs mt-1 line-clamp-2">{reel.description}</p>
              )}

              {/* Stats */}
              <div className="flex items-center gap-3 mt-3">
                <span className="text-xs text-[var(--text-secondary)] flex items-center gap-1">
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
                  </svg>
                  {reel.item_count} item{reel.item_count !== 1 ? 's' : ''}
                </span>
                <span className="text-xs text-[var(--text-secondary)] flex items-center gap-1">
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {formatDuration(reel.total_duration_secs)}
                </span>
              </div>

              {/* Delete button (editor only) */}
              {isEditor && (
                <button
                  onClick={(e) => handleDelete(e, reel)}
                  className="absolute top-3 right-3 p-1.5 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-red-500/20 text-[var(--text-secondary)] hover:text-red-400 transition-all"
                  title="Delete spot reel"
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Create Modal */}
      {showCreate && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-fade-in">
          <div className="bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-2xl w-full max-w-md">
            {/* Header */}
            <div className="p-4 border-b border-[var(--border-color)] flex justify-between items-center">
              <h3 className="text-lg font-bold text-white">Create Spot Reel</h3>
              <button
                onClick={() => setShowCreate(false)}
                className="p-1 hover:bg-[var(--bg-tertiary)] rounded-full text-[var(--text-secondary)] hover:text-white transition-colors"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Form */}
            <div className="p-4 space-y-4">
              <div>
                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Title</label>
                <input
                  type="text"
                  className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                  placeholder="e.g. Commercial Break A"
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  autoFocus
                  onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Description (optional)</label>
                <textarea
                  className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500 resize-none"
                  placeholder="What's this spot reel for?"
                  rows={3}
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                />
              </div>
            </div>

            {/* Footer */}
            <div className="p-4 border-t border-[var(--border-color)] flex justify-end gap-2">
              <button
                onClick={() => setShowCreate(false)}
                className="px-4 py-2 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-white transition-colors text-sm"
              >
                Cancel
              </button>
              <button
                onClick={handleCreate}
                disabled={!newTitle.trim() || creating}
                className="btn-primary disabled:opacity-50"
              >
                {creating ? 'Creating...' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
