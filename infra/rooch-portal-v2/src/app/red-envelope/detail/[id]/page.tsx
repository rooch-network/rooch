import { CONFIG } from 'src/config-global';

import RedEnvelopeDetail from 'src/sections/red-envelope/detail/view';

export const metadata = { title: `Page two | Dashboard - ${CONFIG.site.name}` };

export default function Page({ params }: { params: { id: string } }) {
  return <RedEnvelopeDetail redEnvelopeId={params.id} />;
}
