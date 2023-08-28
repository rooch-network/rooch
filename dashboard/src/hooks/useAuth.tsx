import { useContext } from 'react'
import { AuthContext } from 'src/context/auth/AuthContext'

export const useAuth = () => useContext(AuthContext)
