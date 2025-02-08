import { useNavigate } from 'react-router-dom';
import { Room } from '../types/room';
import { ChatBubbleLeftIcon } from '@heroicons/react/24/outline';

interface RoomListProps {
  rooms: Room[];
  activeRoomId?: string;
}

export function RoomList({ rooms, activeRoomId }: RoomListProps) {
  const navigate = useNavigate();

  return (
    <div className="h-full overflow-y-auto bg-gray-50">
      <div className="p-4">
        <h2 className="text-sm font-semibold text-gray-500 mb-2">CHAT HISTORY</h2>
        <div className="space-y-1">
          {rooms.map((room) => (
            <button
              key={room.id}
              onClick={() => navigate(`/chat/${room.id}`)}
              className={`w-full text-left px-3 py-2 rounded-lg flex items-center gap-2 hover:bg-gray-100 ${
                activeRoomId === room.id ? 'bg-gray-200' : ''
              }`}
            >
              <ChatBubbleLeftIcon className="h-5 w-5 text-gray-400" />
              <div className="truncate flex-1">
                <div className="text-sm font-medium truncate">{room.title}</div>
                <div className="text-xs text-gray-500">
                  {new Date(room.lastActive * 1000).toLocaleDateString()}
                </div>
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}