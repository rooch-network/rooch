import { InvitationsView } from 'src/sections/invitations/index';

export const metadata = { title: `Invitation` };

export default function Page({ params }: { params: { address: string } }) {
  return <InvitationsView />;
}