
import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useQueryClient } from '@tanstack/react-query'
import { DndContext, useSensor, useSensors, PointerSensor, DragEndEvent, DragStartEvent, DragMoveEvent, CollisionDetection, rectIntersection } from '@dnd-kit/core'
import {
  useSchedules,
  useBlocks,
  useCreateBlock,
  useUpdateBlock,
  useUpdateSchedule,
  useDeleteSchedule,
  blocksQueryKey,
} from '../hooks/useSchedules'
import { blocksOverlap } from '../utils/scheduleOverlap'
import { useAuthStore } from '../stores/authStore'
import { ScheduleGrid } from '../components/ScheduleGrid/ScheduleGrid'
import { BlockEditorPopover } from '../components/ScheduleGrid/BlockEditorPopover'
import { OneOffScheduleList } from '../components/ScheduleGrid/OneOffScheduleList'
import { apiClient } from '../api/client'

const fixCursorSnapOffset: CollisionDetection = (args) => {
  if (!args.pointerCoordinates) {
    return rectIntersection(args)
  }
  const { x, y } = args.pointerCoordinates
  const { width, height } = args.collisionRect
  const updated = {
    ...args,
    collisionRect: {
      width,
      height,
      bottom: y + height / 2,
      left: x - width / 2,
      right: x + width / 2,
      top: y - height / 2,
    },
  }
  return rectIntersection(updated)
}

const RESIZE_ID_PREFIX = 'resize-'

// Resize draggables use ids like `resize-42`; move draggables use the numeric
// block id. Returns the block id for resize drags, null otherwise.
const parseResizeId = (activeId: string | number): number | null => {
  const id = String(activeId)
  if (!id.startsWith(RESIZE_ID_PREFIX)) return null
  const parsed = parseInt(id.slice(RESIZE_ID_PREFIX.length), 10)
  return Number.isNaN(parsed) ? null : parsed
}

const timeToMinutes = (time: string): number => {
  const [hours, minutes] = time.split(':').map(Number)
  return hours * 60 + minutes
}

// dnd-kit's delta is cumulative from drag start: snap the vertical delta to
// 15-minute steps and clamp to [15, midnight] relative to the block start.
const computeResizedMinutes = (
  startMinutes: number,
  originalMinutes: number,
  deltaYPx: number,
  pixelsPerMinute: number
): number => {
  const snappedDelta = Math.round(deltaYPx / pixelsPerMinute / 15) * 15
  return Math.max(15, Math.min(originalMinutes + snappedDelta, 24 * 60 - startMinutes))
}

export default function SchedulePage() {
  const { user } = useAuthStore()
  const isEditor = user?.role === 'admin' || user?.role === 'editor'
  const { id } = useParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  // The URL is the single source of truth for schedule selection.
  const routeScheduleId = id ? parseInt(id, 10) : NaN
  const selectedScheduleId = Number.isNaN(routeScheduleId) ? null : routeScheduleId

  const { data: schedules = [] } = useSchedules()
  const { data: blocks = [] } = useBlocks(selectedScheduleId)
  // Mutations are unreachable while selectedScheduleId is null, so the
  // fallback id 0 is never used in a request.
  const createBlockMutation = useCreateBlock(selectedScheduleId ?? 0)
  const updateBlockMutation = useUpdateBlock(selectedScheduleId ?? 0)
  const updateScheduleMutation = useUpdateSchedule()
  const deleteScheduleMutation = useDeleteSchedule()
  const [zoomLevel, setZoomLevel] = useState(2)
  const [selectedBlockId, setSelectedBlockId] = useState<number | null>(null)
  const [resizePreview, setResizePreview] = useState<{ id: number; minutes: number } | null>(null)
  const [popoverPos, setPopoverPos] = useState<{ x: number, y: number } | null>(null)
  const [titleInputValue, setTitleInputValue] = useState('')
  const [timezone, setTimezone] = useState<string>('UTC')

  // Settings aren't covered by the schedule hooks; load timezone once.
  useEffect(() => {
    fetchSettings()
  }, [])

  // Default to first schedule if the URL has no valid schedule id
  useEffect(() => {
    if (!selectedScheduleId && schedules.length > 0) {
      navigate(`/schedules/${schedules[0].id}`, { replace: true })
    }
  }, [selectedScheduleId, schedules, navigate])

  const fetchSettings = async () => {
    try {
      const { data } = await apiClient.get('/api/settings')
      const tz = data.find((s: { key: string }) => s.key === 'timezone')?.value
      if (tz) setTimezone(tz)
    } catch (e) {
      console.error("Failed to fetch settings", e)
    }
  }

  const selectedSchedule = schedules.find(s => s.id === selectedScheduleId)
  const isOneOff = selectedSchedule?.schedule_type === 'one_off'

  // Sync title
  useEffect(() => {
    if (selectedSchedule) {
      setTitleInputValue(selectedSchedule.name)
    }
  }, [selectedScheduleId, schedules])

  const handleTitleSave = async () => {
    if (!isEditor) return
    if (!selectedScheduleId || !titleInputValue.trim()) return
    const schedule = schedules.find(s => s.id === selectedScheduleId)
    if (schedule && schedule.name !== titleInputValue) {
      try {
        await updateScheduleMutation.mutateAsync({ id: selectedScheduleId, data: { name: titleInputValue } })
      } catch (e) {
        console.error("Failed to update schedule name", e)
        setTitleInputValue(schedule.name)
      }
    }
  }

  const handleDeleteSchedule = async () => {
    if (!isEditor) return
    if (!selectedScheduleId) return
    if (window.confirm('Are you sure you want to delete this schedule? This action cannot be undone.')) {
      try {
        await deleteScheduleMutation.mutateAsync(selectedScheduleId)
        navigate('/schedules')
      } catch (e) {
        console.error("Failed to delete schedule", e)
        alert("Failed to delete schedule")
      }
    }
  }

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  )

  const [activeId, setActiveId] = useState<number | null>(null)

  const handleDragStart = (event: DragStartEvent) => {
    if (!isEditor) return
    const resizeId = parseResizeId(event.active.id)
    if (resizeId !== null) {
      // No DragOverlay for resizes; seed the live duration preview instead.
      const block = blocks.find(b => b.id === resizeId)
      if (block) setResizePreview({ id: resizeId, minutes: block.duration_minutes })
      return
    }
    setActiveId(parseInt(String(event.active.id), 10))
  }

  const handleDragMove = (event: DragMoveEvent) => {
    const resizeId = parseResizeId(event.active.id)
    if (resizeId === null) return
    const block = blocks.find(b => b.id === resizeId)
    if (!block) return
    setResizePreview({
      id: resizeId,
      minutes: computeResizedMinutes(
        timeToMinutes(block.start_time),
        block.duration_minutes,
        event.delta.y,
        zoomLevel
      ),
    })
  }

  const handleDragEnd = async (event: DragEndEvent) => {
    if (!isEditor) return
    const { active } = event;
    setActiveId(null);
    setResizePreview(null);
    if (!selectedScheduleId) return;

    const resizeId = parseResizeId(active.id)
    if (resizeId !== null) {
      const block = blocks.find(b => b.id === resizeId);
      if (!block) return;

      const newDuration = computeResizedMinutes(
        timeToMinutes(block.start_time),
        block.duration_minutes,
        event.delta.y,
        zoomLevel
      );
      if (newDuration === block.duration_minutes) return;

      const hasOverlap = blocksOverlap(
        blocks,
        Number(block.day_of_week),
        null,
        block.start_time,
        newDuration,
        block.id
      );
      if (hasOverlap) {
        alert('Cannot resize: Overlaps with an existing block.');
        return;
      }

      try {
        await updateBlockMutation.mutateAsync({
          blockId: resizeId,
          data: {
            ...block,
            duration_minutes: newDuration,
            schedule_id: Number(block.schedule_id),
          },
        });
      } catch (e) {
        console.error("Failed to resize block", e);
        queryClient.invalidateQueries({ queryKey: blocksQueryKey(selectedScheduleId) });
      }
      return;
    }

    const blockId = parseInt(String(active.id), 10);
    const block = blocks.find(b => b.id === blockId);

    if (block) {
      const gridContainer = document.getElementById('schedule-grid-container');
      const activatorEvent = event.activatorEvent as MouseEvent | TouchEvent;
      let clientX = 0;
      let clientY = 0;

      if ('touches' in activatorEvent) {
        clientX = activatorEvent.changedTouches[0].clientX;
        clientY = activatorEvent.changedTouches[0].clientY;
      } else if ('clientX' in activatorEvent) {
        clientX = activatorEvent.clientX;
        clientY = activatorEvent.clientY;
      }

      const finalPointerX = clientX + event.delta.x;
      const finalPointerY = clientY + event.delta.y;

      if (!gridContainer) return;

      const gridRect = gridContainer.getBoundingClientRect();
      const blockRect = active.rect.current.initial;
      if (!blockRect) return;

      const dropCenterX = finalPointerX;
      if (dropCenterX < gridRect.left || dropCenterX > gridRect.right) return;

      const relativeX = dropCenterX - gridRect.left;
      const columnWidth = gridRect.width / 7;
      const newDayIndex = Math.floor(relativeX / columnWidth);

      const dropVisualTop = finalPointerY - (blockRect.height / 2);
      const relativeY = dropVisualTop - gridRect.top;
      const newTotalMinutes = Math.round(relativeY / zoomLevel);
      let snappedMinutes = Math.round(newTotalMinutes / 15) * 15;
      snappedMinutes = Math.max(0, Math.min(24 * 60 - 15, snappedMinutes));

      const newHours = Math.floor(snappedMinutes / 60);
      const newMins = snappedMinutes % 60;
      const newTimeStr = `${newHours.toString().padStart(2, '0')}:${newMins.toString().padStart(2, '0')}:00`;

      const currentDay = Number(block.day_of_week);
      const currentTime = block.start_time;

      if (newDayIndex !== currentDay || newTimeStr !== currentTime) {
        const hasOverlap = blocksOverlap(
          blocks,
          newDayIndex,
          null,
          newTimeStr,
          block.duration_minutes,
          block.id
        );

        if (hasOverlap) {
          alert('Cannot drop here: Overlaps with an existing block.');
          return;
        }

        try {
          await updateBlockMutation.mutateAsync({
            blockId,
            data: {
              ...block,
              day_of_week: Number(newDayIndex),
              start_time: newTimeStr,
              schedule_id: Number(block.schedule_id),
            },
          });
        } catch (e) {
          console.error("Failed to move block", e);
          queryClient.invalidateQueries({ queryKey: blocksQueryKey(selectedScheduleId) });
        }
      }
    }
  }

  const handleGridClick = async (dayIndex: number, startTime: string, e: React.MouseEvent) => {
    if (!selectedScheduleId || !isEditor) return;
    const pos = { x: e.clientX, y: e.clientY };
    setPopoverPos(pos);

    try {
      const hasOverlap = blocksOverlap(
        blocks,
        dayIndex,
        null,
        startTime,
        30
      );

      if (hasOverlap) {
        alert('Cannot create block here: Overlaps with an existing block.');
        return;
      }

      const newBlock = await createBlockMutation.mutateAsync({
        day_of_week: dayIndex,
        start_time: startTime,
        duration_minutes: 30,
        content_id: null
      });

      if (newBlock) {
        setSelectedBlockId(newBlock.id);
      }
    } catch (e) {
      console.error("Failed to create block instantly", e);
    }
  };

  const handleBlockClick = (blockId: number, e: React.MouseEvent) => {
    const pos = { x: e.clientX, y: e.clientY };
    setPopoverPos(pos);
    setSelectedBlockId(blockId);
  }

  return (
    <div className="h-full flex flex-col gap-4 relative overflow-hidden">
      <div className="flex justify-between items-center bg-[var(--bg-secondary)] p-4 rounded-xl border border-[var(--border-color)]">
        <div className="flex items-center gap-4 flex-1">
          <div className="flex-1">
            <input
              type="text"
              className="text-2xl font-bold bg-transparent border-b border-transparent hover:border-[var(--border-color)] focus:border-indigo-500 focus:outline-none text-white w-full transition-colors disabled:opacity-75 disabled:cursor-not-allowed"
              value={titleInputValue}
              onChange={(e) => setTitleInputValue(e.target.value)}
              onBlur={handleTitleSave}
              onKeyDown={(e) => e.key === 'Enter' && handleTitleSave()}
              placeholder="Schedule Name"
              readOnly={!isEditor}
              disabled={!isEditor}
            />
            <p className="text-sm text-[var(--text-secondary)]">
              {isOneOff ? 'Manage your one-off events' : "Manage your channel's programming"}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-4">
          {!isOneOff && (
            <div className="flex items-center gap-2 bg-[var(--bg-primary)] rounded-lg p-1 border border-[var(--border-color)]">
              <button
                onClick={() => setZoomLevel(Math.max(1, zoomLevel - 0.5))}
                className="p-1 hover:bg-[var(--bg-secondary)] rounded text-[var(--text-secondary)] hover:text-white transition-colors"
                title="Zoom Out"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
                </svg>
              </button>
              <span className="text-xs font-medium w-12 text-center text-[var(--text-secondary)]">Zoom</span>
              <button
                onClick={() => setZoomLevel(Math.min(10, zoomLevel + 0.5))}
                className="p-1 hover:bg-[var(--bg-secondary)] rounded text-[var(--text-secondary)] hover:text-white transition-colors"
                title="Zoom In"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
              </button>
            </div>
          )}

          {isEditor && (
            <button
              onClick={handleDeleteSchedule}
              className="px-4 py-2 bg-red-500/10 text-red-400 border border-red-500/20 rounded-lg hover:bg-red-500/20 transition-all duration-200 flex items-center gap-2"
              disabled={!selectedScheduleId}
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
              Delete Schedule
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 min-h-0 flex relative">
        {isOneOff ? (
          <div className="flex-1 w-full relative">
            <OneOffScheduleList />
          </div>
        ) : (
          <>
            <div className="flex-1 w-full relative">
              <DndContext
                sensors={isEditor ? sensors : []}
                onDragStart={handleDragStart}
                onDragMove={handleDragMove}
                onDragEnd={handleDragEnd}
                onDragCancel={() => setResizePreview(null)}
                collisionDetection={fixCursorSnapOffset}
              >
                <ScheduleGrid
                  pixelsPerMinute={zoomLevel}
                  onGridClick={handleGridClick}
                  onBlockClick={handleBlockClick}
                  activeId={activeId}
                  timezone={timezone}
                  readOnly={!isEditor}
                  resizePreview={resizePreview}
                />
              </DndContext>
            </div>

            {selectedBlockId && popoverPos && (
              <BlockEditorPopover
                blockId={selectedBlockId}
                position={popoverPos}
                onClose={() => {
                  setSelectedBlockId(null);
                  setPopoverPos(null);
                }}
                readOnly={!isEditor}
              />
            )}
          </>
        )}
      </div>
    </div>
  )
}
