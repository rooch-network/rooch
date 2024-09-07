import { usePathname } from './use-pathname';
import { hasParams, removeParams, isExternalLink, removeLastSlash } from '../utils';

export function useActiveLink(itemPath: string, deep: boolean = true): boolean {
  const pathname = removeLastSlash(usePathname());

  const pathHasParams = hasParams(itemPath);

  const notValid = itemPath.startsWith('#') || isExternalLink(itemPath);

  if (notValid) {
    return false;
  }

  const isDeep = deep || pathHasParams;

  if (isDeep) {
    /**
     * [1] Deep: default
     * @itemPath 			 = '/dashboard/account'
     * @match pathname = '/dashboard/account'
     * @match pathname = '/dashboard/account/list'
     * @match pathname = '/dashboard/account/tb1pjugffa0n2ts0vra032t3phae7xrehdjfzkg284ymvf260vjh225s5u4z76'
     */
    const defaultActive = pathname.includes(itemPath);

    /**
     * [1] Deep: has params
     * @itemPath 			 = '/dashboard/account?id=tb1pjugffa0n2ts0vra032t3phae7xrehdjfzkg284ymvf260vjh225s5u4z76'
     * @match pathname = '/dashboard/account'
     */

    const originItemPath = removeParams(itemPath);

    const hasParamsActive = pathHasParams && originItemPath === pathname;

    return defaultActive || hasParamsActive;
  }

  /**
   * [1] Normal: active
   * @itemPath 			 = '/dashboard/account'
   * @match pathname = '/dashboard/account'
   */
  return pathname === itemPath;
}
