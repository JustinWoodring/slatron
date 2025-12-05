import { useEffect } from 'react'
import { useScheduleStore } from '../stores/scheduleStore'

export default function SchedulePage() {
  const { schedules, fetchSchedules } = useScheduleStore()

  useEffect(() => {
    fetchSchedules()
  }, [])

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold text-white">Schedules</h1>
        <button className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700">
          Create Schedule
        </button>
      </div>

      <div className="bg-gray-800 shadow overflow-hidden sm:rounded-md">
        <ul className="divide-y divide-gray-700">
          {schedules.length === 0 ? (
            <li className="px-6 py-4 text-gray-400 text-center">
              No schedules yet. Create your first schedule to get started.
            </li>
          ) : (
            schedules.map((schedule) => (
              <li key={schedule.id}>
                <div className="px-4 py-4 sm:px-6 hover:bg-gray-700 cursor-pointer">
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <h3 className="text-lg font-medium text-white">
                        {schedule.name}
                      </h3>
                      {schedule.description && (
                        <p className="mt-1 text-sm text-gray-400">
                          {schedule.description}
                        </p>
                      )}
                      <div className="mt-2 flex items-center text-sm text-gray-400">
                        <span className="mr-4">
                          Type: {schedule.schedule_type}
                        </span>
                        <span className="mr-4">Priority: {schedule.priority}</span>
                        <span
                          className={`px-2 py-1 rounded-full text-xs ${
                            schedule.is_active
                              ? 'bg-green-900 text-green-200'
                              : 'bg-red-900 text-red-200'
                          }`}
                        >
                          {schedule.is_active ? 'Active' : 'Inactive'}
                        </span>
                      </div>
                    </div>
                    <div className="ml-4">
                      <button className="text-indigo-400 hover:text-indigo-300">
                        Edit
                      </button>
                    </div>
                  </div>
                </div>
              </li>
            ))
          )}
        </ul>
      </div>
    </div>
  )
}
