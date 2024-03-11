import { MainContent } from './components/main-content'
import { Navbar } from './components/navbar'
import { Sidebar } from './components/sidebar'

export const DashboardLayout = () => {
  return (
    <div className="h-full scroll-smooth">
      <div className="h-[56px] md:h-[64px] md:pl-60 fixed inset-y-0 w-full z-50">
        <Navbar />
      </div>
      <div className="hidden md:flex h-full w-60 flex-col fixed inset-y-0 z-50">
        <Sidebar onClose={() => {}} />
      </div>
      <main className="md:pl-60 pt-[64px] h-full w-full overflow-y-auto">
        <MainContent />
      </main>
    </div>
  )
}
