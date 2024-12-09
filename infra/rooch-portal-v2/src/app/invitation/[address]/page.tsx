import { InvitationsView } from 'src/sections/invitations/view';

export const metadata = { title: `Srarch Invitation` };

export default function Page({ params }: { params: { address: string } }) {
  return <InvitationsView />;
}
