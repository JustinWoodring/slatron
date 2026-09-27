import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  createPermission,
  deletePermission,
  getPermissions,
  type Permission,
  type PermissionLevel,
  type ResourceType,
} from '../../api/permissions'
import { apiClient } from '../../api/client'

interface PermissionsModalProps {
  user: { id: number; username: string }
  onClose: () => void
}

interface IdNameOption {
  id: number
  name: string
}

interface IdTitleOption {
  id: number
  title: string
}

const RESOURCE_TYPES: { value: ResourceType; label: string; allLabel: string }[] = [
  { value: 'content', label: 'Content', allLabel: 'All content' },
  { value: 'schedule', label: 'Schedule', allLabel: 'All schedules' },
  { value: 'node', label: 'Node', allLabel: 'All nodes' },
]

const LEVELS: { value: PermissionLevel; label: string; help: string }[] = [
  { value: 'read', label: 'Read', help: 'Can view this resource' },
  { value: 'write', label: 'Write', help: 'Can view and edit this resource' },
  { value: 'admin', label: 'Admin', help: 'Full control of this resource' },
]

const allLabelFor = (type: string) =>
  RESOURCE_TYPES.find(t => t.value === type)?.allLabel ?? `All ${type}`

const getLevelColor = (level: string) => {
  switch (level) {
    case 'admin':
      return 'bg-red-900 text-red-200'
    case 'write':
      return 'bg-blue-900 text-blue-200'
    default:
      return 'bg-gray-700 text-gray-300'
  }
}

const extractApiError = (err: unknown, fallback: string): string => {
  const e = err as { response?: { status?: number; data?: unknown } }
  const data = e?.response?.data
  if (typeof data === 'string' && data.length > 0) return data
  if (data && typeof data === 'object' && 'error' in data) {
    const msg = (data as { error: unknown }).error
    if (typeof msg === 'string' && msg.length > 0) return msg
  }
  if (e?.response?.status === 403) return 'Admin access is required to manage permissions.'
  if (e?.response?.status === 400) return 'Invalid permission request.'
  return fallback
}

export default function PermissionsModal({ user, onClose }: PermissionsModalProps) {
  const queryClient = useQueryClient()
  const [resourceType, setResourceType] = useState<ResourceType>('content')
  const [resourceId, setResourceId] = useState(0)
  const [level, setLevel] = useState<PermissionLevel>('read')

  const { data: permissions = [], isLoading: permissionsLoading } = useQuery({
    queryKey: ['permissions'],
    queryFn: getPermissions,
  })
  const { data: content = [] } = useQuery<IdTitleOption[]>({
    queryKey: ['content'],
    queryFn: async () => {
      const { data } = await apiClient.get('/api/content')
      return data
    },
  })
  const { data: schedules = [] } = useQuery<IdNameOption[]>({
    queryKey: ['schedules'],
    queryFn: async () => {
      const { data } = await apiClient.get('/api/schedules')
      return data
    },
  })
  const { data: nodes = [] } = useQuery<IdNameOption[]>({
    queryKey: ['nodes'],
    queryFn: async () => {
      const { data } = await apiClient.get('/api/nodes')
      return data
    },
  })

  const userPermissions = permissions.filter(p => p.user_id === user.id)

  const invalidatePermissions = () => {
    queryClient.invalidateQueries({ queryKey: ['permissions'] })
  }

  const addMutation = useMutation({
    mutationFn: createPermission,
    onSuccess: () => {
      setResourceId(0)
      invalidatePermissions()
    },
  })
  const deleteMutation = useMutation({
    mutationFn: deletePermission,
    onSuccess: invalidatePermissions,
  })

  const apiError = addMutation.error
    ? extractApiError(addMutation.error, 'Failed to add permission')
    : deleteMutation.error
      ? extractApiError(deleteMutation.error, 'Failed to delete permission')
      : null

  const resourceOptions: { id: number; label: string }[] =
    resourceType === 'content'
      ? content.map(c => ({ id: c.id, label: c.title }))
      : resourceType === 'schedule'
        ? schedules.map(s => ({ id: s.id, label: s.name }))
        : nodes.map(n => ({ id: n.id, label: n.name }))

  const resolveResourceLabel = (p: Permission) => {
    if (p.resource_id === 0) return allLabelFor(p.resource_type)
    if (p.resource_type === 'content')
      return content.find(c => c.id === p.resource_id)?.title ?? `Content #${p.resource_id}`
    if (p.resource_type === 'schedule')
      return schedules.find(s => s.id === p.resource_id)?.name ?? `Schedule #${p.resource_id}`
    if (p.resource_type === 'node')
      return nodes.find(n => n.id === p.resource_id)?.name ?? `Node #${p.resource_id}`
    return `#${p.resource_id}`
  }

  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault()
    addMutation.mutate({
      user_id: user.id,
      resource_type: resourceType,
      resource_id: resourceId,
      permission_level: level,
    })
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
      <div className="glass-panel w-full max-w-lg max-h-[85vh] flex flex-col rounded-xl border border-[var(--border-color)] shadow-2xl overflow-hidden">
        <div className="px-6 py-4 border-b border-[var(--border-color)] flex justify-between items-center">
          <div>
            <h3 className="text-lg font-bold text-white">Permissions</h3>
            <p className="text-sm text-[var(--text-secondary)]">{user.username}</p>
          </div>
          <button
            onClick={onClose}
            className="text-[var(--text-secondary)] hover:text-white transition-colors"
            aria-label="Close"
          >
            ✕
          </button>
        </div>

        <div className="p-6 space-y-6 overflow-y-auto">
          {apiError && (
            <div className="p-3 rounded bg-red-500/10 text-red-400 text-sm border border-red-500/20">
              {apiError}
            </div>
          )}

          <div>
            <h4 className="text-sm font-semibold text-white mb-2">Current Grants</h4>
            {permissionsLoading ? (
              <div className="text-center text-[var(--text-secondary)] py-4 text-sm">Loading permissions...</div>
            ) : userPermissions.length === 0 ? (
              <p className="text-sm text-[var(--text-secondary)] py-2">
                No permissions granted yet.
              </p>
            ) : (
              <ul className="space-y-2">
                {userPermissions.map(p => (
                  <li
                    key={p.id}
                    className="flex items-center justify-between px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg"
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="text-sm text-white truncate">
                        {resolveResourceLabel(p)}
                      </span>
                      <span className="text-xs text-[var(--text-secondary)]">
                        {p.resource_type}
                      </span>
                      <span
                        className={`px-2 py-0.5 rounded-full text-xs font-medium ${getLevelColor(p.permission_level)}`}
                      >
                        {p.permission_level}
                      </span>
                    </div>
                    <button
                      onClick={() => deleteMutation.mutate(p.id)}
                      disabled={deleteMutation.isPending}
                      className="text-red-400 hover:text-red-300 px-2 py-1 rounded border border-red-400 text-xs transition-colors disabled:opacity-50"
                    >
                      Delete
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <form onSubmit={handleAdd} className="space-y-4">
            <h4 className="text-sm font-semibold text-white">Add Permission</h4>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                  Resource Type
                </label>
                <select
                  value={resourceType}
                  onChange={e => {
                    setResourceType(e.target.value as ResourceType)
                    setResourceId(0)
                  }}
                  className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                >
                  {RESOURCE_TYPES.map(t => (
                    <option key={t.value} value={t.value}>
                      {t.label}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                  Resource
                </label>
                <select
                  value={resourceId}
                  onChange={e => setResourceId(Number(e.target.value))}
                  className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                >
                  <option value={0}>
                    {RESOURCE_TYPES.find(t => t.value === resourceType)?.allLabel}
                  </option>
                  {resourceOptions.map(o => (
                    <option key={o.id} value={o.id}>
                      {o.label}
                    </option>
                  ))}
                </select>
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1">
                Level
              </label>
              <select
                value={level}
                onChange={e => setLevel(e.target.value as PermissionLevel)}
                className="w-full px-3 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              >
                {LEVELS.map(l => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
              </select>
              <p className="mt-1 text-xs text-[var(--text-secondary)]">
                {LEVELS.find(l => l.value === level)?.help}
              </p>
            </div>
            <div className="flex justify-end">
              <button
                type="submit"
                disabled={addMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg shadow-lg shadow-indigo-500/20 disabled:opacity-50"
              >
                {addMutation.isPending ? 'Adding...' : 'Add Permission'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  )
}
