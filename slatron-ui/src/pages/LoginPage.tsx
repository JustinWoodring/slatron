import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { apiClient } from '../api/client'
import { useAuthStore } from '../stores/authStore'

export default function LoginPage() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const navigate = useNavigate()
  const { setAuth } = useAuthStore()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      const response = await apiClient.post('/api/auth/login', {
        username,
        password,
      })

      setAuth(response.data.token, response.data.user)
      navigate('/')
    } catch (err: any) {
      setError(err.response?.data?.message || 'Login failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-[var(--bg-primary)] pattern-grid">
      <div className="max-w-md w-full p-8 glass-panel rounded-2xl shadow-2xl relative overflow-hidden">
        {/* Glow effect */}
        <div className="absolute top-0 right-0 -mr-16 -mt-16 w-64 h-64 rounded-full bg-indigo-500/10 blur-3xl pointer-events-none" />
        <div className="absolute bottom-0 left-0 -ml-16 -mb-16 w-64 h-64 rounded-full bg-cyan-500/10 blur-3xl pointer-events-none" />

        <div className="relative z-10">
          <div className="text-center mb-10">
            <div className="flex justify-center mb-6">
              <img src="/logo.png" alt="Slatron" className="h-16 w-auto" />
            </div>
            <p className="text-[var(--text-secondary)] text-sm font-medium uppercase tracking-widest">
              Broadcast Automation
            </p>
          </div>

          <form className="space-y-6" onSubmit={handleSubmit}>
            {error && (
              <div className="rounded-lg bg-red-500/10 border border-red-500/20 p-4 animate-shake">
                <div className="text-sm text-red-400 text-center font-medium">{error}</div>
              </div>
            )}

            <div className="space-y-4">
              <div>
                <label htmlFor="username" className="block text-xs font-medium text-[var(--text-secondary)] uppercase mb-1">
                  Username
                </label>
                <input
                  id="username"
                  name="username"
                  type="text"
                  required
                  className="block w-full px-4 py-3 bg-[var(--bg-tertiary)] border border-[var(--border-color)] rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all"
                  placeholder="Enter your username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                />
              </div>

              <div>
                <label htmlFor="password" className="block text-xs font-medium text-[var(--text-secondary)] uppercase mb-1">
                  Password
                </label>
                <input
                  id="password"
                  name="password"
                  type="password"
                  required
                  className="block w-full px-4 py-3 bg-[var(--bg-tertiary)] border border-[var(--border-color)] rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all"
                  placeholder="Enter your password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full btn-primary py-3 text-base flex justify-center relative overflow-hidden group"
            >
              <div className="absolute inset-0 bg-white/10 translate-y-full group-hover:translate-y-0 transition-transform duration-300" />
              <span className="relative">{loading ? 'Authenticating...' : 'Sign In'}</span>
            </button>
          </form>

          <div className="mt-8 text-center">
            <p className="text-xs text-[var(--text-secondary)]">
              System v1.0 • Protected Access
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
