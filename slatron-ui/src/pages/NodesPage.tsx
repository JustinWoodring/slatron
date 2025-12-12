import { useEffect, useState } from 'react'
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  useSortable,
  verticalListSortingStrategy
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { apiClient } from '../api/client'
import { formatDistanceToNow } from 'date-fns'
import { useScheduleStore } from '../stores/scheduleStore'
import { useContentStore } from '../stores/contentStore'
import { useAuthStore } from '../stores/authStore'
import { RegisterNodeModal } from '../components/Nodes/RegisterNodeModal'
import { NodeSettingsModal } from '../components/Nodes/NodeSettingsModal'
import { NodeLogs } from '../components/NodeLogs'

interface Node {
  id: number
  name: string
  status: string
  ip_address: string | null
  last_heartbeat: string | null
  created_at: string
  current_content_id: number | null
  playback_position_secs: number | null
}
interface SortableItemProps {
  id: string
  name: string
  scheduleType: string
  onRemove: (id: string) => void
}

function SortableItem(props: SortableItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition
  } = useSortable({ id: props.id })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      className="bg-[var(--bg-secondary)] p-3 rounded-lg flex items-center justify-between border border-[var(--border-color)] group cursor-move hover:border-indigo-500/50"
    >
      <div className="flex items-center gap-3">
        <svg className="w-4 h-4 text-[var(--text-secondary)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
        </svg>
        <div>
          <div className="text-sm font-medium text-white">{props.name}</div>
          <div className="text-xs text-[var(--text-secondary)]">{props.scheduleType}</div>
        </div>
      </div>
      <button
        type="button"
        onPointerDown={(e) => {
          e.preventDefault()
          e.stopPropagation()
          props.onRemove(props.id)
        }}
        className="text-[var(--text-secondary)] hover:text-red-400 p-1 opacity-0 group-hover:opacity-100 transition-opacity"
      >
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  )
}

import { EffectiveScheduleViewer } from '../components/Nodes/EffectiveScheduleViewer'

export default function NodesPage() {
  const { user } = useAuthStore()
  const isEditor = user?.role === 'admin' || user?.role === 'editor'
  const [nodes, setNodes] = useState<Node[]>([])
  const [loading, setLoading] = useState(true)
  const [assignModalOpen, setAssignModalOpen] = useState(false)
  const [registerModalOpen, setRegisterModalOpen] = useState(false)
  const [settingsModalOpen, setSettingsModalOpen] = useState(false)
  const [selectedNodeId, setSelectedNodeId] = useState<number | null>(null)

  const [viewScheduleModalOpen, setViewScheduleModalOpen] = useState(false)
  const [viewLogsModalOpen, setViewLogsModalOpen] = useState(false)
  const [nodeSchedule, setNodeSchedule] = useState<any>(null)

  const { schedules, fetchSchedules, nodeAssignedSchedules, fetchNodeAssignedSchedules, updateNodeSchedules } = useScheduleStore()
  const { content, fetchContent } = useContentStore()
  const [selectedScheduleId, setSelectedScheduleId] = useState<string>('')

  const [timezone, setTimezone] = useState<string>('UTC')

  useEffect(() => {
    fetchNodes()
    fetchSchedules()
    fetchContent()
    fetchSettings()
    const interval = setInterval(fetchNodes, 5000)
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

  const fetchSettings = async () => {
    try {
      const { data } = await apiClient.get('/api/settings')
      const tz = data.find((s: any) => s.key === 'timezone')?.value
      if (tz) setTimezone(tz)
    } catch (e) {
      console.error("Failed to fetch settings", e)
    }
  }


  const handleViewSchedule = async (nodeId: number) => {
    try {
      const response = await apiClient.get(`/api/nodes/${nodeId}/schedule`)
      setNodeSchedule(response.data)
      setSelectedNodeId(nodeId)
      setViewScheduleModalOpen(true)
    } catch (error) {
      console.error("Failed to fetch node schedule", error)
    }
  }

  const handleViewLogs = (nodeId: number) => {
    setSelectedNodeId(nodeId)
    setViewLogsModalOpen(true)
  }

  const handleSettingsClick = (node: Node) => {
    setSelectedNodeId(node.id)
    setSettingsModalOpen(true)
  }

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  )

  const handleManageSchedules = async (nodeId: number) => {
    setSelectedNodeId(nodeId)
    await fetchNodeAssignedSchedules(nodeId)
    setAssignModalOpen(true)
  }

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (over && active.id !== over.id) {
      useScheduleStore.setState((state) => {
        const oldIndex = state.nodeAssignedSchedules.findIndex((item) => item.id.toString() === active.id)
        const newIndex = state.nodeAssignedSchedules.findIndex((item) => item.id.toString() === over.id)
        return {
          nodeAssignedSchedules: arrayMove(state.nodeAssignedSchedules, oldIndex, newIndex)
        }
      })
    }
  }

  const handleAddSchedule = () => {
    if (!selectedScheduleId) return
    const scheduleToAdd = schedules.find(s => s.id === parseInt(selectedScheduleId))
    if (scheduleToAdd) {
      // Add to start (top) or end? User said "Top of list is more precedence"
      // If we add to bottom, it's lowest priority.
      // Let's add to top.
      useScheduleStore.setState(state => ({
        nodeAssignedSchedules: [scheduleToAdd, ...state.nodeAssignedSchedules]
      }))
      setSelectedScheduleId('')
    }
  }

  const handleRemoveSchedule = (id: string) => {
    useScheduleStore.setState(state => ({
      nodeAssignedSchedules: state.nodeAssignedSchedules.filter(s => s.id.toString() !== id)
    }))
  }

  const handleSaveSchedules = async () => {
    if (selectedNodeId) {
      const ids = nodeAssignedSchedules.map(s => s.id)
      await updateNodeSchedules(selectedNodeId, ids)
      setAssignModalOpen(false)
      fetchNodes() // refreshes status potentially
    }
  }


  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'online':
        return (
          <span className="px-2 py-1 rounded-full text-xs font-semibold bg-green-500/20 text-green-300 border border-green-500/30 flex items-center gap-1.5 w-fit">
            <span className="w-1.5 h-1.5 rounded-full bg-green-400 shadow-[0_0_5px_rgba(74,222,128,0.5)] animate-pulse" />
            ONLINE
          </span>
        )
      case 'offline':
        return (
          <span className="px-2 py-1 rounded-full text-xs font-semibold bg-gray-500/20 text-gray-300 border border-gray-500/30 w-fit">
            OFFLINE
          </span>
        )
      case 'error':
        return (
          <span className="px-2 py-1 rounded-full text-xs font-semibold bg-red-500/20 text-red-300 border border-red-500/30 w-fit">
            ERROR
          </span>
        )
      default:
        return null
    }
  }

  return (
    <div className="space-y-6 relative">
      <div className="flex justify-between items-center bg-[var(--bg-secondary)] p-4 rounded-xl border border-[var(--border-color)]">
        <div>
          <h1 className="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">
            Nodes
          </h1>
          <p className="text-[var(--text-secondary)] mt-1">Manage your playback nodes</p>
        </div>
        {isEditor && (
          <button
            onClick={() => setRegisterModalOpen(true)}
            className="btn-primary flex items-center gap-2"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
            Register Node
          </button>
        )}
      </div>

      <RegisterNodeModal
        isOpen={registerModalOpen}
        onClose={() => setRegisterModalOpen(false)}
        onSuccess={fetchNodes}
      />

      {loading ? (
        <div className="glass-panel rounded-xl p-12 flex flex-col items-center justify-center text-[var(--text-secondary)]">
          <svg className="w-8 h-8 animate-spin mb-4 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Loading nodes...
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4">
          {nodes.length === 0 ? (
            <div className="glass-panel rounded-xl p-12 text-center border-dashed border-2 border-[var(--border-color)]">
              <p className="text-[var(--text-secondary)]">No nodes registered yet.</p>
              {isEditor && (
                <button
                  onClick={() => setRegisterModalOpen(true)}
                  className="mt-4 text-indigo-400 hover:text-indigo-300 font-medium"
                >
                  Register your first node
                </button>
              )}
            </div>
          ) : (
            nodes.map((node) => (
              <div key={node.id} className="glass-panel p-4 rounded-xl flex items-center justify-between group hover:bg-[var(--bg-tertiary)] transition-colors">
                <div className="flex items-center gap-6">
                  <div className="w-12 h-12 rounded-lg bg-[var(--bg-primary)] flex items-center justify-center text-2xl border border-[var(--border-color)]">
                    📺
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="text-lg font-bold text-white group-hover:text-indigo-300 transition-colors truncate">
                      {node.name}
                    </h3>

                    {/* Playback Status */}
                    {node.current_content_id && (
                      <div className="mt-2 text-sm text-[var(--text-secondary)]">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="w-1.5 h-1.5 rounded-full bg-indigo-400 animate-pulse" />
                          <span className="text-indigo-300 font-medium truncate">
                            {content.find(c => c.id === node.current_content_id)?.title || 'Unknown Content'}
                          </span>
                        </div>
                        {node.playback_position_secs !== null && (
                          <div className="w-full h-1 bg-[var(--bg-primary)] rounded-full overflow-hidden">
                            <div
                              className="h-full bg-indigo-500 transition-all duration-1000 ease-linear"
                              style={{
                                width: `${Math.min(100, (node.playback_position_secs / ((content.find(c => c.id === node.current_content_id)?.duration_minutes || 0) * 60)) * 100)}%`
                              }}
                            />
                          </div>
                        )}
                      </div>
                    )}

                    <div className="flex items-center gap-4 mt-2 text-xs text-[var(--text-secondary)]">
                      {node.ip_address && (
                        <span className="flex items-center gap-1 font-mono">
                          <svg className="w-3 h-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                          </svg>
                          {node.ip_address}
                        </span>
                      )}
                      {node.last_heartbeat && (
                        <span className="flex items-center gap-1">
                          <svg className="w-3 h-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                          </svg>
                          Seen {formatDistanceToNow(new Date(node.last_heartbeat))} ago
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-6">
                  {getStatusBadge(node.status)}
                  <div className="h-8 w-px bg-[var(--border-color)]" />
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleViewSchedule(node.id)}
                      className="px-3 py-1.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-sm hover:border-indigo-500 hover:text-indigo-400 transition-colors"
                    >
                      Schedule
                    </button>
                    <button
                      onClick={() => handleViewLogs(node.id)}
                      className="px-3 py-1.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-sm hover:border-indigo-500 hover:text-indigo-400 transition-colors"
                    >
                      Logs
                    </button>
                    {isEditor && (
                      <>
                        <button
                          onClick={() => handleManageSchedules(node.id)}
                          className="px-3 py-1.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-sm hover:border-indigo-500 hover:text-indigo-400 transition-colors"
                        >
                          Manage
                        </button>
                        <button
                          onClick={() => handleSettingsClick(node)}
                          className="px-3 py-1.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-sm hover:border-indigo-500 hover:text-indigo-400 transition-colors"
                        >
                          Settings
                        </button>
                      </>
                    )}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Node Settings Modal */}
      <NodeSettingsModal
        isOpen={settingsModalOpen}
        onClose={() => setSettingsModalOpen(false)}
        node={nodes.find(n => n.id === selectedNodeId) || null}
        onSuccess={fetchNodes}
      />

      {/* Assign Schedule Modal */}
      {assignModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="glass-panel p-6 rounded-xl w-full max-w-sm border border-[var(--border-color)]">
            <h2 className="text-xl font-bold text-white mb-4">Manage Schedules</h2>

            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragEnd={handleDragEnd}
            >
              <SortableContext
                items={nodeAssignedSchedules.map(s => s.id.toString())}
                strategy={verticalListSortingStrategy}
              >
                <div className="space-y-2 mb-6 max-h-[300px] overflow-y-auto pr-1">
                  {nodeAssignedSchedules.length === 0 && (
                    <p className="text-center text-[var(--text-secondary)] italic py-4">No schedules assigned.</p>
                  )}
                  {nodeAssignedSchedules.map((schedule) => (
                    <SortableItem
                      key={schedule.id}
                      id={schedule.id.toString()}
                      name={schedule.name}
                      scheduleType={schedule.schedule_type}
                      onRemove={handleRemoveSchedule}
                    />
                  ))}
                </div>
              </SortableContext>
            </DndContext>

            <div className="border-t border-[var(--border-color)] pt-4 space-y-4">
              <div className="flex gap-2">
                <select
                  className="flex-1 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-lg p-2 text-white focus:outline-none focus:border-indigo-500 text-sm"
                  value={selectedScheduleId}
                  onChange={(e) => setSelectedScheduleId(e.target.value)}
                >
                  <option value="">Add Schedule...</option>
                  {schedules
                    .filter(s => !nodeAssignedSchedules.find(as => as.id === s.id))
                    .map(s => (
                      <option key={s.id} value={s.id}>{s.name} ({s.schedule_type})</option>
                    ))}
                </select>
                <button
                  onClick={handleAddSchedule}
                  disabled={!selectedScheduleId}
                  className="px-3 py-1 rounded-lg bg-indigo-500/20 text-indigo-300 hover:bg-indigo-500/30 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Add
                </button>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setAssignModalOpen(false)}
                  className="flex-1 px-4 py-2 rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:bg-[var(--bg-secondary)] transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSaveSchedules}
                  className="flex-1 btn-primary"
                >
                  Save Order
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* View Schedule Modal */}
      {viewScheduleModalOpen && nodeSchedule && (
        <EffectiveScheduleViewer
          data={nodeSchedule}
          timezone={timezone}
          onClose={() => setViewScheduleModalOpen(false)}
        />
      )}

      {/* View Logs Modal */}
      {viewLogsModalOpen && selectedNodeId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="glass-panel p-6 rounded-xl w-full max-w-4xl border border-[var(--border-color)] max-h-[80vh] flex flex-col">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-bold text-white">Node Logs</h2>
            </div>

            <NodeLogs nodeId={selectedNodeId} />

            <div className="flex justify-end pt-4">
              <button onClick={() => setViewLogsModalOpen(false)} className="btn-secondary">Close</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
