import { RoochAddress, Session } from '@roochnetwork/rooch-sdk'

export function getInitialValidSession(
  accountAddress: RoochAddress,
  sessions: Session[],
): Session | null {
  const matched = sessions
    .filter(session => !session.isExpired())
    .find(session => session.getRoochAddress().toStr() === accountAddress.toStr())

  return matched || null
}
