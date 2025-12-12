import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useScheduleStore } from '../stores/scheduleStore'
import { useAuthStore } from '../stores/authStore'
import { CreateScheduleModal } from '../components/Schedules/CreateScheduleModal'

export default function SchedulesListPage() {
    const { user } = useAuthStore()
    const isEditor = user?.role === 'admin' || user?.role === 'editor'
    const { schedules, fetchSchedules } = useScheduleStore()
    const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
    const navigate = useNavigate()

    useEffect(() => {
        fetchSchedules()
    }, [])

    return (
        <div className="h-full flex flex-col gap-6 p-6">
            <div className="flex justify-between items-center">
                <div>
                    <h1 className="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">
                        Schedules
                    </h1>
                    <p className="text-[var(--text-secondary)] mt-1">Manage your channel programming schedules</p>
                </div>
                {isEditor && (
                    <button
                        onClick={() => setIsCreateModalOpen(true)}
                        className="btn-primary flex items-center gap-2"
                    >
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                        </svg>
                        Create Schedule
                    </button>
                )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {schedules.map(schedule => (
                    <div
                        key={schedule.id}
                        onClick={() => navigate(`/schedules/${schedule.id}`)}
                        className="group bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl p-5 cursor-pointer hover:border-indigo-500/50 hover:shadow-lg hover:shadow-indigo-500/10 transition-all duration-200"
                    >
                        <div className="flex justify-between items-start mb-3">
                            <div className="h-10 w-10 rounded-lg bg-indigo-500/10 flex items-center justify-center text-indigo-400 group-hover:bg-indigo-500 group-hover:text-white transition-colors duration-200">
                                <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                </svg>
                            </div>
                            <span className={`px-2 py-1 rounded text-xs font-medium ${schedule.is_active ? 'bg-green-500/10 text-green-400' : 'bg-gray-500/10 text-gray-400'}`}>
                                {schedule.is_active ? 'Active' : 'Draft'}
                            </span>
                        </div>

                        <h3 className="text-lg font-bold text-white mb-1 group-hover:text-indigo-400 transition-colors">{schedule.name}</h3>
                        <p className="text-sm text-[var(--text-secondary)] line-clamp-2">
                            {schedule.description || "No description"}
                        </p>

                        <div className="mt-4 pt-4 border-t border-[var(--border-color)] flex justify-between items-center text-xs text-[var(--text-secondary)]">
                            <span>ID: {schedule.id}</span>
                            <span>Type: {schedule.schedule_type}</span>
                        </div>
                    </div>
                ))}

                {/* Create New Card (optional visual cue) */}
                {isEditor && (
                    <div
                        onClick={() => setIsCreateModalOpen(true)}
                        className="border-2 border-dashed border-[var(--border-color)] rounded-xl p-5 flex flex-col items-center justify-center text-[var(--text-secondary)] hover:border-indigo-500/50 hover:text-indigo-400 hover:bg-[var(--bg-secondary)]/30 transition-all cursor-pointer min-h-[180px]"
                    >
                        <div className="h-12 w-12 rounded-full bg-[var(--bg-secondary)] flex items-center justify-center mb-3">
                            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                            </svg>
                        </div>
                        <span className="font-medium">Create New Schedule</span>
                    </div>
                )}
            </div>

            <CreateScheduleModal
                isOpen={isCreateModalOpen}
                onClose={() => setIsCreateModalOpen(false)}
            />
        </div>
    )
}
