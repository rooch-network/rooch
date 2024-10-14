'use client';

export function isMainNetwork() {
  return (
    window.location.hostname === 'portal.rooch.network' ||
    window.location.hostname === 'main-portal.rooch.network'
  );
}
