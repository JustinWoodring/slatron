import { useState } from 'react'
import { useContent, useDeleteContent, useUpdateContent, ContentItem } from '../hooks/useContent'
import { useAuthStore } from '../stores/authStore'
import CreateContentModal from '../components/Content/CreateContentModal'
import ContentPreviewModal from '../components/Content/ContentPreviewModal'

export default function ContentPage() {
  const { user } = useAuthStore()
  const isEditor = user?.role === 'admin' || user?.role === 'editor'
  const { data: content = [] } = useContent()
  const deleteContentMutation = useDeleteContent()
  const updateContentMutation = useUpdateContent()
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [editingContent, setEditingContent] = useState<ContentItem | undefined>(undefined)
  const [previewItem, setPreviewItem] = useState<ContentItem | undefined>(undefined)
  const [searchQuery, setSearchQuery] = useState('')
  const [typeFilter, setTypeFilter] = useState('')
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid')

  const contentTypes = content
    .map((item) => item.content_type)
    .filter((type, index, all) => all.indexOf(type) === index)
    .sort()
  const searchLower = searchQuery.trim().toLowerCase()
  const filteredContent = content.filter((item) => {
    if (typeFilter && item.content_type !== typeFilter) return false
    if (!searchLower) return true
    return (
      item.title.toLowerCase().includes(searchLower) ||
      (item.description ?? '').toLowerCase().includes(searchLower) ||
      (item.tags ?? '').toLowerCase().includes(searchLower)
    )
  })

  const handleEdit = (item: ContentItem) => {
    setEditingContent(item)
    setIsModalOpen(true)
  }

  const handleDelete = async (id: number) => {
    if (window.confirm('Are you sure you want to delete this content?')) {
      try {
        await deleteContentMutation.mutateAsync(id)
      } catch (error) {
        alert('Failed to delete content')
      }
    }
  }

  const handleToggleDjAccess = async (id: number, current: boolean) => {
    try {
      await updateContentMutation.mutateAsync({ id, updates: { is_dj_accessible: !current } })
    } catch (error) {
      console.error("Failed to toggle DJ access", error)
    }
  }

  const handleClose = () => {
    setIsModalOpen(false)
    setEditingContent(undefined)
  }

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-2xl font-bold bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">Content Library</h1>
          <p className="text-sm text-[var(--text-secondary)]">Manage your media assets</p>
        </div>
        <div className="flex items-center gap-3">
          {/* View Toggle */}
          <div className="flex bg-[var(--bg-secondary)] rounded-lg p-1 border border-[var(--border-color)]">
            <button
              onClick={() => setViewMode('grid')}
              className={`p-1.5 rounded ${viewMode === 'grid' ? 'bg-indigo-600 text-white' : 'text-[var(--text-secondary)] hover:text-white'}`}
              title="Grid View"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
              </svg>
            </button>
            <button
              onClick={() => setViewMode('list')}
              className={`p-1.5 rounded ${viewMode === 'list' ? 'bg-indigo-600 text-white' : 'text-[var(--text-secondary)] hover:text-white'}`}
              title="List View"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
          </div>

          {isEditor && (
            <button
              onClick={() => {
                setEditingContent(undefined)
                setIsModalOpen(true)
              }}
              className="bg-indigo-600 text-white px-4 py-2 rounded-lg hover:bg-indigo-700 transition-colors shadow-lg shadow-indigo-500/20 font-medium text-sm"
            >
              Add Content
            </button>
          )}
        </div>
      </div>

      {content.length > 0 && (
        <div className="flex flex-col sm:flex-row sm:items-center gap-3 mb-4">
          <div className="relative flex-1">
            <svg className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search title, description, or tags..."
              className="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg pl-9 pr-3 py-2 text-white text-sm focus:border-indigo-500 outline-none"
            />
          </div>
          <select
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value)}
            className="bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-2 text-white text-sm focus:border-indigo-500 outline-none"
          >
            <option value="">All types</option>
            {contentTypes.map((type) => (
              <option key={type} value={type}>{type.replace('_', ' ')}</option>
            ))}
          </select>
          <span className="text-xs text-[var(--text-secondary)] whitespace-nowrap">
            {filteredContent.length} of {content.length} items
          </span>
        </div>
      )}

      {content.length === 0 ? (
        <div className="col-span-full flex flex-col items-center justify-center p-12 border-2 border-dashed border-[var(--border-color)] rounded-xl text-[var(--text-secondary)]">
          <svg className="w-12 h-12 mb-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
          </svg>
          <p>No content items yet</p>
          {isEditor && (
            <button onClick={() => setIsModalOpen(true)} className="mt-2 text-indigo-400 hover:text-indigo-300 text-sm">Add your first item</button>
          )}
        </div>
      ) : filteredContent.length === 0 ? (
        <div className="col-span-full flex flex-col items-center justify-center p-12 border-2 border-dashed border-[var(--border-color)] rounded-xl text-[var(--text-secondary)]">
          <svg className="w-12 h-12 mb-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <p>No content matches your filters</p>
          <button
            onClick={() => {
              setSearchQuery('')
              setTypeFilter('')
            }}
            className="mt-2 text-indigo-400 hover:text-indigo-300 text-sm"
          >
            Clear filters
          </button>
        </div>
      ) : viewMode === 'list' ? (
        // LIST VIEW
        <div className="glass-panel border border-[var(--border-color)] rounded-xl overflow-hidden">
          <table className="w-full text-left border-collapse">
            <thead className="bg-[#0f1115]/50 text-xs uppercase text-[var(--text-secondary)] font-semibold border-b border-[var(--border-color)]">
              <tr>
                <th className="px-4 py-3 w-10"></th> {/* Type Icon */}
                <th className="px-4 py-3">Title</th>
                <th className="px-4 py-3 w-28">Duration</th>
                <th className="px-4 py-3">Path</th>
                <th className="px-4 py-3 w-24 text-center">DJ Access</th>
                <th className="px-4 py-3 w-28 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--border-color)]">
              {filteredContent.map((item) => (
                <tr key={item.id} className="hover:bg-white/5 transition-colors text-sm group">
                  <td className="px-4 py-2 text-center">
                    {/* Type Indicator */}
                    <div className={`w-2 h-2 rounded-full mx-auto ${item.content_type === 'local_file' ? 'bg-emerald-500' :
                        item.content_type === 'remote_url' ? 'bg-blue-500' : 'bg-amber-500'
                      }`} title={item.content_type.replace('_', ' ')} />
                  </td>
                  <td className="px-4 py-2">
                    <div className="font-medium text-white truncate max-w-xs sm:max-w-sm md:max-w-md" title={item.title}>
                      {item.title}
                    </div>
                    {item.description && <div className="text-[10px] text-[var(--text-secondary)] truncate max-w-xs">{item.description}</div>}
                  </td>
                  <td className="px-4 py-2 text-[var(--text-secondary)] font-mono text-xs">
                    {item.duration_minutes ? `${item.duration_minutes}m` : '-'}
                  </td>
                  <td className="px-4 py-2 text-[var(--text-secondary)] text-xs font-mono truncate max-w-[150px]" title={item.content_path}>
                    {item.content_path}
                  </td>
                  <td className="px-4 py-2 text-center">
                    <input
                      type="checkbox"
                      className="rounded bg-gray-700 border-gray-600 text-indigo-600 focus:ring-indigo-500 focus:ring-offset-gray-900 cursor-pointer"
                      checked={item.is_dj_accessible}
                      onChange={() => handleToggleDjAccess(item.id, item.is_dj_accessible)}
                      disabled={!isEditor}
                    />
                  </td>
                  <td className="px-4 py-2 text-right">
                    <div className="flex justify-end items-center gap-2">
                      <button onClick={() => setPreviewItem(item)} className="text-cyan-400 hover:text-cyan-300" title="Preview">
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                      </button>
                      {isEditor && (
                        <div className="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button onClick={() => handleEdit(item)} className="text-indigo-400 hover:text-indigo-300">
                            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                            </svg>
                          </button>
                          <button onClick={() => handleDelete(item.id)} className="text-red-400 hover:text-red-300">
                            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                          </button>
                        </div>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        // GRID VIEW (Existing)
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {filteredContent.map((item) => (
            <div
              key={item.id}
              className="glass-panel border border-[var(--border-color)] rounded-xl p-4 hover:border-indigo-500/50 transition-colors group relative overflow-hidden"
            >
              <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-2">
                {/* Add checkbox to Grid view too for consistency? Might be cluttered. Keep it in List view for now as requested. */}
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    setPreviewItem(item)
                  }}
                  className="p-1.5 bg-cyan-500 text-white rounded-lg hover:bg-cyan-600 shadow-lg"
                  title="Preview"
                >
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                </button>
                {isEditor && (
                  <>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleEdit(item)
                      }}
                      className="p-1.5 bg-indigo-500 text-white rounded-lg hover:bg-indigo-600 shadow-lg"
                      title="Edit"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleDelete(item.id)
                      }}
                      className="p-1.5 bg-red-500 text-white rounded-lg hover:bg-red-600 shadow-lg"
                      title="Delete"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </>
                )}
              </div>

              <div className="mb-2">
                <span className={`text-[10px] uppercase tracking-wider font-bold px-2 py-1 rounded-full ${item.content_type === 'local_file' ? 'bg-emerald-500/10 text-emerald-400' :
                  item.content_type === 'remote_url' ? 'bg-blue-500/10 text-blue-400' :
                    'bg-amber-500/10 text-amber-400'
                  }`}>
                  {item.content_type.replace('_', ' ')}
                </span>
              </div>

              <h3 className="text-white font-medium truncate mb-1" title={item.title}>
                {item.title}
              </h3>

              {item.description && (
                <p className="text-xs text-[var(--text-secondary)] line-clamp-2 mb-3 h-8">
                  {item.description}
                </p>
              )}

              <div className="flex items-center justify-between mt-4 pt-3 border-t border-[var(--border-color)]">
                <div className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {item.duration_minutes ? `${item.duration_minutes} min` : 'N/A'}
                </div>
                <div className="text-[10px] text-[var(--text-secondary)] font-mono opacity-50 truncate max-w-[120px]" title={item.content_path}>
                  {item.content_path}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      <CreateContentModal
        isOpen={isModalOpen}
        onClose={handleClose}
        editingContent={editingContent}
      />

      <ContentPreviewModal
        item={previewItem}
        onClose={() => setPreviewItem(undefined)}
      />
    </div>
  )
}
