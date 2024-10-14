import { FaucetView } from 'src/sections/faucet/view';

export const metadata = { title: `Faucet` };

export default function Page({ params }: { params: { address: string } }) {
  return <FaucetView address={params.address} />;
}
