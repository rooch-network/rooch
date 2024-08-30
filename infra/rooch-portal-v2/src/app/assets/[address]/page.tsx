import { AssetsView } from 'src/sections/assets/view';

export const metadata = { title: `Assets` };

export default function Page({ params }: { params: { address: string } }) {
  return <AssetsView address={params.address} />;
}
