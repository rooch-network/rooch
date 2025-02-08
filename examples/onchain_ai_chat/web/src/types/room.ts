import { bcs } from "@roochnetwork/rooch-sdk";

export interface Room {
  id: string;
  title: string;
  isPublic: boolean;
  creator: string;
  createdAt: number;
  lastActive: number;
  messages: Message[];
}

export interface Message {
  sender: string;
  content: string;
  timestamp: number;
  messageType: number; // 0 for user, 1 for AI
}

export const MessageSchema = bcs.struct('Message', {
  sender: bcs.Address,
  content: bcs.String,
  timestamp: bcs.U64,
  messageType: bcs.U8,
})

export function transformMessage(messageData: Map<string, any>): Array<Message> {
  if (!messageData || !(messageData instanceof Map)) {
    return [];
  }

  return Array.from(messageData.values()).map(field => {
    return {
      sender: field.value.sender,
      content: field.value.content,
      timestamp: field.value.timestamp,
      messageType: field.value.messageType,
    } as Message;
  });
}