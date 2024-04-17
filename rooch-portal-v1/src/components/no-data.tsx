import { useTheme } from './theme-provider'

import { Table, TableBody, TableCell, TableRow } from '@/components/ui/table'

export const NoData = () => {
  const { theme } = useTheme()

  const logoSrc = theme === 'dark' ? '/rooch_white_logo.svg' : '/rooch_black_logo.svg'

  return (
    <div className="rounded-lg border w-full overflow-hidden">
      <Table>
        <TableBody>
          <TableRow>
            <TableCell>
              <div className="flex justify-center items-center flex-col" style={{ height: '80vh' }}>
                <img src={logoSrc} alt="No Data" style={{ width: '200px', height: '200px' }} />
                <p className="text-gray-500 mt-4">No data found :(</p>
              </div>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  )
}
