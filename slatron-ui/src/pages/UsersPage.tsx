import { useEffect, useState } from 'react'
import { apiClient } from '../api/client'
import { formatDistanceToNow } from 'date-fns'

interface User {
  id: number
  username: string
  role: string
  created_at: string
}

export default function UsersPage() {
  const [users, setUsers] = useState<User[]>([])
  const [loading, setLoading] = useState(true)
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [modalMode, setModalMode] = useState<'create' | 'edit'>('create')
  const [selectedUser, setSelectedUser] = useState<User | null>(null)
  const [formData, setFormData] = useState({ username: '', password: '', role: 'viewer' })
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchUsers()
  }, [])

  const fetchUsers = async () => {
    try {
      const response = await apiClient.get('/api/users')
      setUsers(response.data)
    } catch (error) {
      console.error('Failed to fetch users:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleCreate = () => {
    setModalMode('create')
    setFormData({ username: '', password: '', role: 'viewer' })
    setSelectedUser(null)
    setIsModalOpen(true)
    setError(null)
  }

  const handleEdit = (user: User) => {
    setModalMode('edit')
    setFormData({ username: user.username, password: '', role: user.role })
    setSelectedUser(user)
    setIsModalOpen(true)
    setError(null)
  }

  const handleDelete = async (id: number) => {
    if (!confirm('Are you sure you want to delete this user?')) return
    try {
      await apiClient.delete(`/api/users/${id}`)
      fetchUsers()
    } catch (err) {
      alert('Failed to delete user')
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      if (modalMode === 'create') {
        if (!formData.password) {
          setError("Password is required for new users")
          return
        }
        await apiClient.post('/api/users', formData)
      } else {
        // For edit, only send password if changed
        const payload: any = { username: formData.username, role: formData.role }
        if (formData.password) {
          payload.password = formData.password
        }
        await apiClient.put(`/api/users/${selectedUser!.id}`, payload)
      }
      setIsModalOpen(false)
      fetchUsers()
    } catch (err: any) {
      setError(err.response?.data || 'Operation failed')
    }
  }

  const getRoleColor = (role: string) => {
    switch (role) {
      case 'admin':
        return 'bg-red-900 text-red-200'
      case 'editor':
        return 'bg-blue-900 text-blue-200'
      case 'viewer':
        return 'bg-gray-700 text-gray-300'
      default:
        return 'bg-gray-700 text-gray-300'
    }
  }

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold text-white">Users</h1>
        <button
          onClick={handleCreate}
          className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700"
        >
          Create User
        </button>
      </div>

      {loading ? (
        <div className="text-center text-gray-400 py-8">Loading...</div>
      ) : (
        <div className="bg-[var(--bg-secondary)] shadow overflow-hidden sm:rounded-md border border-[var(--border-color)]">
          <ul className="divide-y divide-[var(--border-color)]">
            {users.length === 0 ? (
              <li className="px-6 py-4 text-gray-400 text-center">
                No users found.
              </li>
            ) : (
              users.map((user) => (
                <li key={user.id}>
                  <div className="px-4 py-4 sm:px-6">
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <div className="flex items-center">
                          <h3 className="text-lg font-medium text-white mr-3">
                            {user.username}
                          </h3>
                          <span
                            className={`px-2 py-1 rounded-full text-xs font-medium ${getRoleColor(
                              user.role
                            )}`}
                          >
                            {user.role}
                          </span>
                        </div>
                        <p className="mt-1 text-sm text-gray-400">
                          Created {formatDistanceToNow(new Date(user.created_at))}{' '}
                          ago
                        </p>
                      </div>
                      <div className="ml-4 flex gap-2">
                        <button
                          onClick={() => handleEdit(user)}
                          className="text-indigo-400 hover:text-indigo-300 px-3 py-1 rounded border border-indigo-400 transition-colors"
                        >
                          Edit
                        </button>
                        {user.username !== 'admin' && (
                          <button
                            onClick={() => handleDelete(user.id)}
                            className="text-red-400 hover:text-red-300 px-3 py-1 rounded border border-red-400 transition-colors"
                          >
                            Delete
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                </li>
              ))
            )}
          </ul>
        </div>
      )}

      {/* Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
          <div className="w-full max-w-md bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-2xl overflow-hidden">
            <div className="px-6 py-4 border-b border-[var(--border-color)]">
              <h3 className="text-lg font-bold text-white">
                {modalMode === 'create' ? 'Create User' : 'Edit User'}
              </h3>
            </div>
            <form onSubmit={handleSubmit} className="p-6 space-y-4">
              {error && (
                <div className="p-3 rounded bg-red-500/10 text-red-400 text-sm border border-red-500/20">
                  {error}
                </div>
              )}
              <div>
                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Username</label>
                <input
                  type="text"
                  required
                  value={formData.username}
                  onChange={e => setFormData({ ...formData, username: e.target.value })}
                  className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                  {modalMode === 'create' ? 'Password' : 'New Password (leave blank to keep)'}
                </label>
                <input
                  type="password"
                  value={formData.password}
                  onChange={e => setFormData({ ...formData, password: e.target.value })}
                  className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">Role</label>
                <select
                  value={formData.role}
                  onChange={e => setFormData({ ...formData, role: e.target.value })}
                  className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                >
                  <option value="viewer">Viewer</option>
                  <option value="editor">Editor</option>
                  <option value="admin">Admin</option>
                </select>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  type="button"
                  onClick={() => setIsModalOpen(false)}
                  className="px-4 py-2 text-sm font-medium text-[var(--text-secondary)] hover:text-white"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg shadow-lg shadow-indigo-500/20"
                >
                  Save
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
