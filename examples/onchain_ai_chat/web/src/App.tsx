import { useState } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { Theme} from "@radix-ui/themes";
import { Home } from './pages/Home'
import { Room } from './pages/Room'

import "@roochnetwork/rooch-sdk-kit/dist/index.css";
import '@radix-ui/themes/styles.css'


function App() {
  return (
          <Theme>
            <Router>
              <Routes>
                <Route path="/" element={<Home />} />
                <Route path="/chat/:roomId" element={<Room />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </Router>
          </Theme>
  )
}

export default App
