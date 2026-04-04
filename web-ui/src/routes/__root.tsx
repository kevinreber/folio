import { Outlet } from '@tanstack/react-router'
import { Sidebar } from '../components/Sidebar'

export function RootLayout() {
  return (
    <div id="app">
      <Sidebar />
      <main id="content">
        <Outlet />
      </main>
    </div>
  )
}
