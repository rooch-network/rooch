import { bcs } from '@roochnetwork/rooch-sdk'

export interface Room {
  id: string;
  title: string;
  is_public: boolean;
  creator: string;
  created_at: number;
  last_active: number;
  status: number;
  room_type: number;
  message_counter: number;
}

export interface Message {
  id: number;
  sender: string;
  content: string;
  timestamp: number;
  message_type: number; // 0 for user, 1 for AI
}

export const MessageSchema = bcs.struct('Message', {
  id: bcs.u64(),
  sender: bcs.Address,
  content: bcs.string(),
  timestamp: bcs.u64(),
  message_type: bcs.u8(),
})

export const RoomSchema = bcs.struct('Room', {
  id: bcs.String,
  title: bcs.String,
  is_public: bcs.Bool,
  creator: bcs.Address,
  created_at: bcs.U64,
  last_active: bcs.U64,
  status: bcs.U8,
  room_type: bcs.U8,
});