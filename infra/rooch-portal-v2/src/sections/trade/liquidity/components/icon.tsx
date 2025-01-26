import DOMPurify from 'dompurify';

import { Box } from '@mui/material';

import { Iconify } from 'src/components/iconify';

export default function Icon({ url }: { url?: string }) {
  return url ? (
    <Box
      component="span"
      className="mr-1"
      sx={{ width: 32, height: 32 }}
      dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(url) }}
    />
  ) : (
    <Iconify className="mr-1" icon="solar:question-circle-line-duotone" width={32} height={32} />
  );
}
