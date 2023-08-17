// ** React Imports
import { useEffect, useState } from 'react'

// ** Axios Import
import axios from 'axios'

// ** Type Import
import { VerticalNavItemsType } from 'src/@core/layouts/types'

const ServerSideNavItems = () => {
  // ** State
  const [menuItems, setMenuItems] = useState<VerticalNavItemsType>([])

  useEffect(() => {
    axios.get('/api/vertical-nav/data').then(response => {
      const menuArray = response.data

      setMenuItems(menuArray)
    })
  }, [])

  return { menuItems }
}

export default ServerSideNavItems
