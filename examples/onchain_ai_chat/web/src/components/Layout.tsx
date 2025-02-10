import { ReactNode } from 'react';
import { ConnectButton } from '@roochnetwork/rooch-sdk-kit';
import { RoomListContainer } from '../containers/RoomListContainer';
import { ErrorGuard } from '../ErrorGuard';
import { Link } from 'react-router-dom';
import { OracleBalance } from './OracleBalance';

interface LayoutProps {
  children: ReactNode;
  showRoomList?: boolean;
}

export function Layout({ children, showRoomList = false }: LayoutProps) {
  return (
    <div className="h-screen flex flex-col">
      <header className="flex-none flex items-center justify-between px-6 h-16 border-b bg-white">
        <Link to="/" className="text-2xl font-bold text-gray-900 hover:text-gray-700 transition-colors">
          OnChain AI Chat
        </Link>
        <ConnectButton />
      </header>
      
      <div className="flex-1 flex min-h-0 bg-gray-50">
        {showRoomList && (
          <aside className="w-64 flex-none overflow-y-auto border-r bg-white">
            <RoomListContainer />
            <ErrorGuard />
          </aside>
        )}
        <main className="flex-1 overflow-auto">
          {children}
        </main>
      </div>
      
      <OracleBalance />
    </div>
  );
}