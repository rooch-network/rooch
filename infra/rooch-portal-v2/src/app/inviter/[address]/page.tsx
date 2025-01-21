
import { InviterView } from 'src/sections/inviter/index';

export default function Page({ params }: { params: { address: string } }) {
  return <InviterView inviterAddress={params.address} />;
}
