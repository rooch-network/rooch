'use client';

import { m } from 'framer-motion';

import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { PageNotFoundIllustration } from 'src/assets/illustrations';

import { varBounce, MotionContainer } from 'src/components/animate';

export function NotFoundView() {
  return (
    <Container component={MotionContainer} className="w-full justify-center items-center mt-24">
      <m.div variants={varBounce().in}>
        <Typography variant="h3" sx={{ mb: 2 }} className="text-center">
          Sorry, page not found!
        </Typography>
      </m.div>

      <m.div variants={varBounce().in}>
        <Typography sx={{ color: 'text.secondary' }}>
          {/* Sorry, we couldn’t find the page you’re looking for. Perhaps you’ve mistyped the URL? Be
            sure to check your spelling. */}
        </Typography>
      </m.div>

      <m.div variants={varBounce().in} className="flex justify-center">
        <PageNotFoundIllustration sx={{ my: { xs: 5, sm: 10 } }} />
      </m.div>

      {/* <Button component={RouterLink} href="/" size="large" variant="contained">
          Go to home
        </Button> */}
    </Container>
  );
}
