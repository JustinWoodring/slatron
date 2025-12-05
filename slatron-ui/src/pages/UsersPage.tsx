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
        <button className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700">
          Create User
        </button>
      </div>

      {loading ? (
        <div className="text-center text-gray-400 py-8">Loading...</div>
      ) : (
        <div className="bg-gray-800 shadow overflow-hidden sm:rounded-md">
          <ul className="divide-y divide-gray-700">
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
                        <button className="text-indigo-400 hover:text-indigo-300 px-3 py-1 rounded border border-indigo-400">
                          Edit
                        </button>
                        {user.username !== 'admin' && (
                          <button className="text-red-400 hover:text-red-300 px-3 py-1 rounded border border-red-400">
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
    </div>
  )
}
