import { TransactionsView } from 'src/sections/transactions/view';

export const metadata = { title: `Transactions` };

export default function Page({ params }: { params: { address: string } }) {
  return <TransactionsView address={params.address} />;
}
