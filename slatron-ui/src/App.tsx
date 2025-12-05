import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from './stores/authStore'
import LoginPage from './pages/LoginPage'
import DashboardPage from './pages/DashboardPage'
import SchedulePage from './pages/SchedulePage'
import ContentPage from './pages/ContentPage'
import NodesPage from './pages/NodesPage'
import ScriptsPage from './pages/ScriptsPage'
import UsersPage from './pages/UsersPage'
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
        <Route path="schedules" element={<SchedulePage />} />
        <Route path="content" element={<ContentPage />} />
        <Route path="nodes" element={<NodesPage />} />
        <Route path="scripts" element={<ScriptsPage />} />
        <Route path="users" element={<UsersPage />} />
      </Route>
    </Routes>
  )
}

export default App
