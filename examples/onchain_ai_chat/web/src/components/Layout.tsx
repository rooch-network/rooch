import { ReactNode } from 'react';
import { ConnectButton } from '@roochnetwork/rooch-sdk-kit';

interface LayoutProps {
  children: ReactNode;
  showRoomList?: boolean;
}

export function Layout({ children, showRoomList = false }: LayoutProps) {
  return (
    <div className="h-screen flex flex-col bg-white">
      <header className="flex items-center justify-between px-6 h-16 border-b bg-white">
        <h1 className="text-2xl font-bold text-gray-900">AI Chat</h1>
        <ConnectButton />
      </header>
      <div className="flex-1 flex overflow-hidden">
        {showRoomList && (
          <aside className="w-64 border-r bg-gray-50">
            {/* RoomList component will be added here */}
          </aside>
        )}
        <main className="flex-1 overflow-y-auto">
          {children}
        </main>
      </div>
    </div>
  );
}