import { useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'
import { makeMoveProof } from '@rooch/zklogin'

const publicURL = `${import.meta.env.VITE_PUBLIC_URL}`

function App() {
  const [count, setCount] = useState(0)

  const handleClick = async ()=>{
    setCount((count) => count + 1)

    const moveInfo = await makeMoveProof(publicURL, 1111, 2222, {
      x: 50, y: 50
    }, 51, 51)
    
    console.log("moveInfo:", moveInfo)
  }

  return (
    <>
      <div>
        <a href="https://vitejs.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
        <button onClick={handleClick}>
          count is {count}
        </button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  )
}

export default App
