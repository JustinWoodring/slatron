import { useEffect, useState } from 'react'
import { apiClient } from '../api/client'

interface Script {
  id: number
  name: string
  description: string | null
  script_type: string
  is_builtin: boolean
  created_at: string
}

export default function ScriptsPage() {
  const [scripts, setScripts] = useState<Script[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchScripts()
  }, [])

  const fetchScripts = async () => {
    try {
      const response = await apiClient.get('/api/scripts')
      setScripts(response.data)
    } catch (error) {
      console.error('Failed to fetch scripts:', error)
    } finally {
      setLoading(false)
    }
  }

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'content_loader':
        return 'bg-blue-900 text-blue-200'
      case 'overlay':
        return 'bg-purple-900 text-purple-200'
      case 'global':
        return 'bg-green-900 text-green-200'
      default:
        return 'bg-gray-700 text-gray-300'
    }
  }

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold text-white">Scripts</h1>
        <button className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700">
          Create Script
        </button>
      </div>

      {loading ? (
        <div className="text-center text-gray-400 py-8">Loading...</div>
      ) : (
        <div className="bg-gray-800 shadow overflow-hidden sm:rounded-md">
          <ul className="divide-y divide-gray-700">
            {scripts.length === 0 ? (
              <li className="px-6 py-4 text-gray-400 text-center">
                No custom scripts yet. Builtin scripts are always available.
              </li>
            ) : (
              scripts.map((script) => (
                <li key={script.id}>
                  <div className="px-4 py-4 sm:px-6 hover:bg-gray-700 cursor-pointer">
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <div className="flex items-center">
                          <h3 className="text-lg font-medium text-white mr-3">
                            {script.name}
                          </h3>
                          <span
                            className={`px-2 py-1 rounded-full text-xs font-medium ${getTypeColor(
                              script.script_type
                            )}`}
                          >
                            {script.script_type}
                          </span>
                          {script.is_builtin && (
                            <span className="ml-2 px-2 py-1 rounded-full text-xs font-medium bg-yellow-900 text-yellow-200">
                              Builtin
                            </span>
                          )}
                        </div>
                        {script.description && (
                          <p className="mt-1 text-sm text-gray-400">
                            {script.description}
                          </p>
                        )}
                      </div>
                      <div className="ml-4">
                        {!script.is_builtin && (
                          <button className="text-indigo-400 hover:text-indigo-300">
                            Edit
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
