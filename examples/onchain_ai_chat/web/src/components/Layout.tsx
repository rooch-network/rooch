import { ReactNode } from 'react';
import { ConnectButton } from '@roochnetwork/rooch-sdk-kit';
import { RoomListContainer } from '../containers/RoomListContainer';
import { ErrorGuard } from '../ErrorGuard';

interface LayoutProps {
  children: ReactNode;
  showRoomList?: boolean;
}

export function Layout({ children, showRoomList = false }: LayoutProps) {
  return (
    <div className="min-h-screen flex flex-col bg-white">
      <header className="flex-none flex items-center justify-between px-6 h-16 border-b bg-white">
        <h1 className="text-2xl font-bold text-gray-900">AI Chat</h1>
        <ConnectButton />
      </header>
      <div className="flex-1 flex min-h-0">
        {showRoomList && (
          <aside className="w-64 flex-none overflow-y-auto border-r bg-gray-50">
            <RoomListContainer />
            <ErrorGuard />
          </aside>
        )}
        <main className="flex-1 min-w-0 overflow-hidden flex flex-col">
          {children}
        </main>
      </div>
    </div>
  );
}