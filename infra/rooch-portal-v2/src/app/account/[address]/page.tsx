import { AccountView } from 'src/sections/account/view';

export const metadata = { title: `Account` };

export default function Page({ params }: { params: { address: string } }) {
  return <AccountView address={params.address} />;
}
