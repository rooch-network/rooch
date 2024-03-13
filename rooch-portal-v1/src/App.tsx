import { DashboardLayout } from './(dashboard)/dashboard-layout'
import { ThemeProvider } from '@/components/theme-provider'

function App() {
  return (
    <>
      <ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
        <DashboardLayout />
      </ThemeProvider>
    </>
  )
}

export default App
