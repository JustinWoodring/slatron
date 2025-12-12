import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '../stores/authStore'
import { useScriptStore } from '../stores/scriptStore'
import { CreateScriptModal } from '../components/Scripts/CreateScriptModal'

export default function ScriptsPage() {
  const { user } = useAuthStore()
  const isEditor = user?.role === 'admin' || user?.role === 'editor'
  const { scripts, fetchScripts } = useScriptStore()
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
  const navigate = useNavigate()

  useEffect(() => {
    fetchScripts()
  }, [])

  return (
    <div className="h-full flex flex-col gap-6 p-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold bg-gradient-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">
            Scripts
          </h1>
          <p className="text-[var(--text-secondary)] mt-1">Manage automation scripts and content loaders</p>
        </div>
        {isEditor && (
          <button
            onClick={() => setIsCreateModalOpen(true)}
            className="btn-primary flex items-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
            Create Script
          </button>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {scripts.map((script: any) => (
          <div
            key={script.id}
            onClick={() => navigate(`/scripts/${script.id}`)}
            className="group bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl p-5 cursor-pointer hover:border-emerald-500/50 hover:shadow-lg hover:shadow-emerald-500/10 transition-all duration-200"
          >
            <div className="flex justify-between items-start mb-3">
              <div className="h-10 w-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400 group-hover:bg-emerald-500 group-hover:text-white transition-colors duration-200">
                <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                </svg>
              </div>
              {script.is_builtin && (
                <span className="px-2 py-1 rounded text-xs font-medium bg-blue-500/10 text-blue-400">
                  Built-in
                </span>
              )}
            </div>

            <h3 className="text-lg font-bold text-white mb-1 group-hover:text-emerald-400 transition-colors">{script.name}</h3>
            <p className="text-sm text-[var(--text-secondary)] line-clamp-2">
              {script.description || "No description"}
            </p>

            <div className="mt-4 pt-4 border-t border-[var(--border-color)] flex justify-between items-center text-xs text-[var(--text-secondary)]">
              <span className="capitalize px-2 py-0.5 rounded-full bg-white/5">{script.script_type}</span>
            </div>
          </div>
        ))}
      </div>

      <CreateScriptModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
      />
    </div>
  )
}
