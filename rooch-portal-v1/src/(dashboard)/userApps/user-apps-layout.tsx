import { UserAppItem } from './components/user-app-item'

const mockApps = [
  {
    id: 1,
    name: 'App One',
    description: 'Description for App One.',
    profileUrl:
      'https://cdn.lxdao.io/bafkreig3psglqxqiejrcokqwcoucbv4i2nkp4rumqawok2vjvhey5ps63i.png',
    logoUrl: 'https://cdn.lxdao.io/bafybeibietdc7lxki2jeggdu5namnyisuujhgej2zsq26nn7orn2cngm6y.png',
    type: 'Tag',
  },
  {
    id: 2,
    name: 'App Two',
    description: 'Description for App Two.',
    profileUrl:
      'https://cdn.lxdao.io/bafkreib5gpyab5fipyk7mvs3sbbcophl2gwpldoal3mt7hwzxgbu6pdjpq.png',
    logoUrl: 'https://cdn.lxdao.io/bafkreifmpi4vszs4zqvm25us2omgpfr6gkxmc7cwvmle6xph6d5axsm4jm.png',
    type: 'Bridge',
  },
  {
    id: 3,
    name: 'App One',
    description: 'Description for App One.',
    profileUrl:
      'https://cdn.lxdao.io/bafkreig3psglqxqiejrcokqwcoucbv4i2nkp4rumqawok2vjvhey5ps63i.png',
    logoUrl: 'https://cdn.lxdao.io/bafybeibietdc7lxki2jeggdu5namnyisuujhgej2zsq26nn7orn2cngm6y.png',
    type: 'Game',
  },
  {
    id: 4,
    name: 'App One',
    description: 'Description for App One.',
    profileUrl:
      'https://cdn.lxdao.io/bafkreig3psglqxqiejrcokqwcoucbv4i2nkp4rumqawok2vjvhey5ps63i.png',
    logoUrl: 'https://cdn.lxdao.io/bafybeibietdc7lxki2jeggdu5namnyisuujhgej2zsq26nn7orn2cngm6y.png',
    type: 'Tag',
  },
  {
    id: 5,
    name: 'App Two',
    description: 'Description for App Two.',
    profileUrl:
      'https://cdn.lxdao.io/bafkreib5gpyab5fipyk7mvs3sbbcophl2gwpldoal3mt7hwzxgbu6pdjpq.png',
    logoUrl: 'https://cdn.lxdao.io/bafkreifmpi4vszs4zqvm25us2omgpfr6gkxmc7cwvmle6xph6d5axsm4jm.png',
    type: 'Bridge',
  },
  {
    id: 6,
    name: 'App One',
    description: 'Description for App One.',
    profileUrl:
      'https://cdn.lxdao.io/bafkreig3psglqxqiejrcokqwcoucbv4i2nkp4rumqawok2vjvhey5ps63i.png',
    logoUrl: 'https://cdn.lxdao.io/bafybeibietdc7lxki2jeggdu5namnyisuujhgej2zsq26nn7orn2cngm6y.png',
    type: 'Game',
  },
]

export const UserAppsLayout = () => {
  return (
    <div className="h-full flex-1 flex-col space-y-6 flex">
      <div className="flex items-center justify-between space-y-2">
        <span>
          <h1 className="text-3xl font-bold tracking-tight">Apps</h1>
          <p className="text-muted-foreground text-wrap">
            Explore a variety of apps supported by Rooch, enhancing your Bitcoin Layer-2 experience.
          </p>
        </span>
      </div>
      {/* UserAppItem */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 w-full place-items-center">
        {mockApps.map((app) => (
          <UserAppItem
            key={app.id}
            id={app.id}
            name={app.name}
            description={app.description}
            profileUrl={app.profileUrl}
            logoUrl={app.logoUrl}
            type={app.type}
          />
        ))}
      </div>
    </div>
  )
}
