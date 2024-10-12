export function isMainNetwork() {
  console.log(window.location.hostname)
  return window.location.hostname === 'portal.rooch.network' || window.location.hostname === 'main-portal.rooch.network';
}
