import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../api/client';
import { LiveNodeGrid } from '../components/Dashboard/LiveNodeGrid';

interface Node {
  id: number;
  status: string;
  name: string;
  current_content_id: number | null;
  playback_position_secs: number | null;
  last_heartbeat: string | null;
}

interface ContentItem {
  id: number;
  title: string;
  duration_minutes: number;
}

interface Schedule {
  id: number;
  is_active: boolean;
}

export default function DashboardPage() {
  const { data: nodes = [] } = useQuery<Node[]>({
    queryKey: ['nodes'],
    queryFn: async () => {
      const { data } = await apiClient.get('/api/nodes');
      return data;
    },
    refetchInterval: 5000,
  });

  const { data: content = [] } = useQuery<ContentItem[]>({
    queryKey: ['content'],
    queryFn: async () => {
      const { data } = await apiClient.get('/api/content');
      return data;
    },
    refetchInterval: 10000,
  });

  const { data: schedules = [] } = useQuery<Schedule[]>({
    queryKey: ['schedules'],
    queryFn: async () => {
      const { data } = await apiClient.get('/api/schedules');
      return data;
    },
  });

  const activeSchedulesCount = schedules.filter(s => s.is_active).length;
  const contentCount = content.length;
  const onlineNodesCount = nodes.filter(n => n.status === 'online').length;

  const stats = [
    { label: 'Active Schedules', value: activeSchedulesCount.toString(), icon: '📅', color: 'text-blue-400' },
    { label: 'Content Items', value: contentCount.toString(), icon: '🎬', color: 'text-purple-400' },
    { label: 'Nodes Online', value: onlineNodesCount.toString(), icon: '🟢', color: 'text-green-400' },
  ];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-black bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent mb-2">
          Dashboard
        </h1>
        <p className="text-[var(--text-secondary)]">Overview of your broadcast network</p>
      </div>

      {/* Stats Row */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {stats.map((stat) => (
          <div key={stat.label} className="glass-panel p-6 rounded-xl relative overflow-hidden group hover:bg-[var(--bg-tertiary)] transition-colors">
            <div className="relative z-10 flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-[var(--text-secondary)] uppercase tracking-wider">
                  {stat.label}
                </p>
                <p className="mt-2 text-3xl font-bold text-white group-hover:scale-105 transition-transform origin-left">
                  {stat.value}
                </p>
              </div>
              <span className={`text-4xl opacity-50 ${stat.color}`}>{stat.icon}</span>
            </div>
            {/* Background decoration */}
            <div className="absolute -right-4 -bottom-4 w-24 h-24 bg-gradient-to-br from-indigo-500/10 to-transparent rounded-full blur-2xl group-hover:from-indigo-500/20 transition-all" />
          </div>
        ))}
      </div>

      {/* Live Monitor Grid */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <span className="w-2 h-2 bg-red-500 rounded-full animate-pulse shadow-[0_0_10px_rgba(239,68,68,0.5)]"></span>
            Live Network Monitor
          </h2>
          <span className="text-xs text-[var(--text-secondary)] uppercase tracking-wider">Real-time Telemetry</span>
        </div>

        {nodes.length > 0 ? (
          <LiveNodeGrid nodes={nodes} content={content} />
        ) : (
          <div className="glass-panel rounded-xl p-12 text-center border-dashed border-2 border-[var(--border-color)]">
            <p className="text-[var(--text-secondary)]">No nodes online. Register a node to see live status.</p>
          </div>
        )}
      </div>

      {/* System Status (Simplified) */}
      <div className="glass-panel rounded-xl p-6 border border-[var(--border-color)] flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-white">System Health</h3>
          <p className="text-xs text-[var(--text-secondary)] mt-1">Server and database connectivity</p>
        </div>
        <div className="flex items-center gap-3 bg-[var(--bg-primary)]/50 px-4 py-2 rounded-lg border border-[var(--border-color)]">
          <div className="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.5)] animate-pulse" />
          <span className="font-mono text-xs text-green-400">OPERATIONAL</span>
        </div>
      </div>
    </div>
  )
}
