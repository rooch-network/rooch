import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { useRoochClient, useRoochClientQuery } from '@roochnetwork/rooch-sdk-kit';
import { useNetworkVariable } from '../networks';
import { RoomList } from '../components/RoomList';
import { Room } from '../types/room';

export function RoomListContainer() {
  const { roomId } = useParams<{ roomId: string }>();
  const client = useRoochClient();
  const packageId = useNetworkVariable('packageId');
  const [rooms, setRooms] = useState<Room[]>([]);

  const { data: roomsResponse } = useRoochClientQuery(
    'queryObjectStates',
    {
      filter: {
        object_type: `${packageId}::room::Room`,
      },
    },
    {
      enabled: !!client && !!packageId,
    }
  );

  useEffect(() => {
    if (roomsResponse?.data) {
      try {
        const rooms = roomsResponse.data.map(obj => {
          const roomData = obj.decoded_value.value;
          return {
            id: obj.id,
            title: roomData.title,
            is_public: roomData.is_public,
            creator: roomData.creator,
            created_at: parseInt(roomData.created_at),
            last_active: parseInt(roomData.last_active),
            status: roomData.status,
            room_type: roomData.room_type,
            message_counter: parseInt(roomData.message_counter),
          };
        });
        
        // Sort rooms by last_active timestamp (newest first)
        const sortedRooms = rooms.sort((a, b) => b.last_active - a.last_active);
        setRooms(sortedRooms);
      } catch (error) {
        console.error('Failed to parse rooms:', error);
      }
    }
  }, [roomsResponse]);

  return <RoomList rooms={rooms} activeRoomId={roomId} />;
}