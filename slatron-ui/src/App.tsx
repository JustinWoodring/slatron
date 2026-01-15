import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from './stores/authStore'
import LoginPage from './pages/LoginPage'
import DashboardPage from './pages/DashboardPage'
import SchedulesListPage from './pages/SchedulesListPage'
import SchedulePage from './pages/SchedulePage'
import ContentPage from './pages/ContentPage'
import NodesPage from './pages/NodesPage'
import ScriptsPage from './pages/ScriptsPage'
import ScriptEditorPage from './pages/ScriptEditorPage'
import UsersPage from './pages/UsersPage'
import SettingsPage from './pages/SettingsPage'
import DjsPage from './pages/DjsPage'
import BumpersPage from './pages/BumpersPage'
import Layout from './components/Layout/Layout'

function PrivateRoute({ children }: { children: React.ReactNode }) {
  const { token } = useAuthStore()
  return token ? <>{children}</> : <Navigate to="/login" />
}

function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        path="/"
        element={
          <PrivateRoute>
            <Layout />
          </PrivateRoute>
        }
      >
        <Route index element={<DashboardPage />} />
        <Route path="schedules" element={<SchedulesListPage />} />
        <Route path="schedules/:id" element={<SchedulePage />} />
        <Route path="content" element={<ContentPage />} />
        <Route path="nodes" element={<NodesPage />} />
        <Route path="scripts" element={<ScriptsPage />} />
        <Route path="scripts/:id" element={<ScriptEditorPage />} />
        <Route path="users" element={<UsersPage />} />
        <Route path="djs" element={<DjsPage />} />
        <Route path="bumpers" element={<BumpersPage />} />
        <Route path="settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  )
}

export default App
