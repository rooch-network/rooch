import { TxView } from 'src/sections/tx/view';

export const metadata = { title: `Transaction Detail` };

export default function Page({ params }: { params: { hash: string } }) {
  return <TxView hash={params.hash} />;
}
