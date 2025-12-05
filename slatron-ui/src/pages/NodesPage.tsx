import { useEffect, useState } from 'react'
import { apiClient } from '../api/client'
import { formatDistanceToNow } from 'date-fns'

interface Node {
  id: number
  name: string
  status: string
  ip_address: string | null
  last_heartbeat: string | null
  created_at: string
}

export default function NodesPage() {
  const [nodes, setNodes] = useState<Node[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchNodes()
    const interval = setInterval(fetchNodes, 5000) // Refresh every 5 seconds
    return () => clearInterval(interval)
  }, [])

  const fetchNodes = async () => {
    try {
      const response = await apiClient.get('/api/nodes')
      setNodes(response.data)
    } catch (error) {
      console.error('Failed to fetch nodes:', error)
    } finally {
      setLoading(false)
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online':
        return 'bg-green-900 text-green-200'
      case 'offline':
        return 'bg-gray-700 text-gray-300'
      case 'error':
        return 'bg-red-900 text-red-200'
      default:
        return 'bg-gray-700 text-gray-300'
    }
  }

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold text-white">Nodes</h1>
        <button className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700">
          Register Node
        </button>
      </div>

      {loading ? (
        <div className="text-center text-gray-400 py-8">Loading...</div>
      ) : (
        <div className="bg-gray-800 shadow overflow-hidden sm:rounded-md">
          <ul className="divide-y divide-gray-700">
            {nodes.length === 0 ? (
              <li className="px-6 py-4 text-gray-400 text-center">
                No nodes registered yet. Register your first node to get started.
              </li>
            ) : (
              nodes.map((node) => (
                <li key={node.id}>
                  <div className="px-4 py-4 sm:px-6">
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <div className="flex items-center">
                          <h3 className="text-lg font-medium text-white mr-3">
                            {node.name}
                          </h3>
                          <span
                            className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(
                              node.status
                            )}`}
                          >
                            {node.status}
                          </span>
                        </div>
                        <div className="mt-2 flex items-center text-sm text-gray-400">
                          {node.ip_address && (
                            <span className="mr-4">IP: {node.ip_address}</span>
                          )}
                          {node.last_heartbeat && (
                            <span>
                              Last seen:{' '}
                              {formatDistanceToNow(new Date(node.last_heartbeat))}{' '}
                              ago
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="ml-4 flex gap-2">
                        <button className="text-indigo-400 hover:text-indigo-300 px-3 py-1 rounded border border-indigo-400">
                          Control
                        </button>
                        <button className="text-red-400 hover:text-red-300 px-3 py-1 rounded border border-red-400">
                          Remove
                        </button>
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
