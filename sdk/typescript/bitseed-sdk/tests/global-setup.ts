import { Buffer } from 'buffer';

export default async function global_setup() {
  global.Buffer = Buffer;
}