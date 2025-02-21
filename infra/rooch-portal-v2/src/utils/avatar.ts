export function getAvatar(obj: any): string {
  return 'LogoURL' in obj ? obj.LogoURL : getXAvatar(obj.Twitter);
}

export function getXAvatar(xUrl: string): string {
  if (xUrl.endsWith('/')) {
    xUrl = xUrl.slice(0, -1);
  }

  return `https://unavatar.io/x/${xUrl.slice(xUrl.lastIndexOf('/') + 1)}`;
}
