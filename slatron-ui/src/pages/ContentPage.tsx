import { useEffect, useState } from 'react'
import { useContentStore, ContentItem } from '../stores/contentStore'
import { useAuthStore } from '../stores/authStore'
import CreateContentModal from '../components/Content/CreateContentModal'

export default function ContentPage() {
  const { user } = useAuthStore()
  const isEditor = user?.role === 'admin' || user?.role === 'editor'
  const { content, fetchContent, deleteContent } = useContentStore()
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [editingContent, setEditingContent] = useState<ContentItem | undefined>(undefined)

  const handleEdit = (item: ContentItem) => {
    setEditingContent(item)
    setIsModalOpen(true)
  }

  const handleDelete = async (id: number) => {
    if (window.confirm('Are you sure you want to delete this content?')) {
      try {
        await deleteContent(id)
      } catch (error) {
        alert('Failed to delete content')
      }
    }
  }

  const handleClose = () => {
    setIsModalOpen(false)
    setEditingContent(undefined)
  }

  useEffect(() => {
    fetchContent()
  }, [])

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-2xl font-bold bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">Content Library</h1>
          <p className="text-sm text-[var(--text-secondary)]">Manage your media assets</p>
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

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
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
        ) : (
          content.map((item) => (
            <div
              key={item.id}
              className="glass-panel border border-[var(--border-color)] rounded-xl p-4 hover:border-indigo-500/50 transition-colors group relative overflow-hidden"
            >
              {isEditor && (
                <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-2">
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
                </div>
              )}

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
          ))
        )}
      </div>

      <CreateContentModal
        isOpen={isModalOpen}
        onClose={handleClose}
        editingContent={editingContent}
      />
    </div>
  )
}
