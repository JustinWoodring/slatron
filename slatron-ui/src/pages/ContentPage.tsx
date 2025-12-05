import { useEffect, useState } from 'react'
import { apiClient } from '../api/client'

interface ContentItem {
  id: number
  title: string
  description: string | null
  content_type: string
  content_path: string
  duration_minutes: number | null
  created_at: string
}

export default function ContentPage() {
  const [content, setContent] = useState<ContentItem[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchContent()
  }, [])

  const fetchContent = async () => {
    try {
      const response = await apiClient.get('/api/content')
      setContent(response.data)
    } catch (error) {
      console.error('Failed to fetch content:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold text-white">Content Library</h1>
        <button className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700">
          Add Content
        </button>
      </div>

      {loading ? (
        <div className="text-center text-gray-400 py-8">Loading...</div>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {content.length === 0 ? (
            <div className="col-span-full text-center text-gray-400 py-8">
              No content items yet. Add your first content to get started.
            </div>
          ) : (
            content.map((item) => (
              <div
                key={item.id}
                className="bg-gray-800 rounded-lg shadow p-4 hover:bg-gray-700 cursor-pointer"
              >
                <h3 className="text-lg font-medium text-white truncate">
                  {item.title}
                </h3>
                {item.description && (
                  <p className="mt-1 text-sm text-gray-400 line-clamp-2">
                    {item.description}
                  </p>
                )}
                <div className="mt-3 flex items-center justify-between text-xs text-gray-400">
                  <span className="bg-gray-700 px-2 py-1 rounded">
                    {item.content_type}
                  </span>
                  {item.duration_minutes && (
                    <span>{item.duration_minutes} min</span>
                  )}
                </div>
                <div className="mt-2 text-xs text-gray-500 truncate">
                  {item.content_path}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  )
}
