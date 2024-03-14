import { ConnectedAccount } from './components/connected-account'
import { ConnectedSites } from './components/connected-sites'

export const UserSettingsLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex rounded-lg md:shadow-custom md:p-4 md:dark:shadow-muted">
      {/* Connected Account section */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-4">
          <span>
            <h1 className="text-3xl font-bold tracking-tight">Connected Account</h1>
            <p className="text-muted-foreground text-wrap">Manage and view your accounts.</p>
          </span>
        </div>
        <ConnectedAccount />
      </div>

      {/* Connected Sites section */}
      <div>
        <div className="flex items-center justify-between space-y-2 mb-4">
          <span>
            <h1 className="text-3xl font-bold tracking-tight">Connected Sites</h1>
            <p className="text-muted-foreground text-wrap">
              Account {} is connected to these sites. They can view your account address.
            </p>
          </span>
        </div>
        <ConnectedSites />
      </div>
    </div>
  )
}
