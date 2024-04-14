import { MainContent } from '../components/main-content'
import { Navbar } from '../components/navbar'
import { Sidebar } from '../components/sidebar'
import { Banner } from '../components/banner'
import { useState } from 'react'

export const DashboardLayout = () => {
  const [isBannerVisible, setIsBannerVisible] = useState(true)

  const handleCloseBanner = () => {
    setIsBannerVisible(false)
  }

  return (
    <div className="h-full scroll-smooth font-sans">
      {isBannerVisible && <Banner onClose={handleCloseBanner} />}
      <div
        className={`fixed inset-x-0 top-0 w-full z-40 ${isBannerVisible ? 'pt-[38px]' : 'pt-0'}`}
      >
        <Navbar />
      </div>
      <div
        className={`hidden md:flex h-full w-60 flex-col fixed inset-y-0 z-50 ${
          isBannerVisible ? 'pt-[38px]' : 'pt-0'
        }`}
      >
        <Sidebar onClose={() => {}} />
      </div>
      <main
        className={`md:pl-60 h-full w-full overflow-y-auto ${isBannerVisible ? 'pt-28' : ' pt-20'}`}
      >
        <MainContent />
      </main>
    </div>
  )
}
