import type { RenderEvent } from './types';
import type { activitiesFilter } from '../view';

export const EVENT_MAP = {
  BuyEvent: 'Buy' as const,
  ListEvent: 'List' as const,
  CreateBidEvent: 'Create Bid' as const,
  CancelListEvent: 'Cancel List' as const,
  CancelBidEvent: 'Cancel Bid' as const,
  AcceptBidEvent: 'Accept Bid' as const,
};

export enum EVENT_ENUM {
  BuyEvent = 'Buy',
  ListEvent = 'List',
  CancelListEvent = 'Cancel List',
  CancelBidEvent = 'Cancel Bid',
  CreateBidEvent = 'Create Bid',
  AcceptBidEvent = 'Accept Bid',
}

export function parseEvent(eventType: string) {
  const splitType = eventType.split('::');
  if (
    splitType[2] === 'BuyEvent' ||
    splitType[2] === 'ListedEvent' ||
    splitType[2] === 'CreateBidEvent' ||
    splitType[2] === 'DeListedEvent' ||
    splitType[2] === 'BurnFloorEvent' ||
    splitType[2] === 'AcceptBidEvent'
  ) {
    return splitType[2] as
      | 'BuyEvent'
      | 'ListedEvent'
      | 'CreateBidEvent'
      | 'DeListedEvent'
      | 'BurnFloorEvent'
      | 'AcceptBidEvent';
  }
  return undefined;
}

export const ORDER_TYPE_MAP = {
  0: 'List',
  1: 'Create Bid',
  2: 'Cancel Bid',
  3: 'Cancel List',
  4: 'Buy',
  5: 'Accept Bid',
};

export function isAfter(startDate: Date | null, endDate: Date | null) {
  const results =
    startDate && endDate ? new Date(startDate).getTime() > new Date(endDate).getTime() : false;

  return results;
}

export function emptyRows(page: number, rowsPerPage: number, arrayLength: number) {
  return page ? Math.max(0, (1 + page) * rowsPerPage - arrayLength) : 0;
}

function descendingComparator<T>(a: T, b: T, orderBy: keyof T) {
  if (a[orderBy] === null) {
    return 1;
  }
  if (b[orderBy] === null) {
    return -1;
  }
  if (b[orderBy]! < a[orderBy]!) {
    return -1;
  }
  if (b[orderBy]! > a[orderBy]!) {
    return 1;
  }
  return 0;
}

export function getComparator<Key extends keyof any>(
  order: 'asc' | 'desc',
  orderBy: Key
): (a: { [key in Key]: number | string }, b: { [key in Key]: number | string }) => number {
  return order === 'desc'
    ? (a, b) => descendingComparator(a, b, orderBy)
    : (a, b) => -descendingComparator(a, b, orderBy);
}

export function applyFilter({
  inputData,
  comparator,
  filters,
  dateError,
}: {
  inputData: RenderEvent[];
  comparator: (a: any, b: any) => number;
  filters: {
    type: activitiesFilter;
  };
  dateError: boolean;
}) {
  const { type } = filters;

  const stabilizedThis = inputData.map((el, index) => [el, index] as const);

  stabilizedThis.sort((a, b) => {
    const order = comparator(a[0], b[0]);
    if (order !== 0) return order;
    return a[1] - b[1];
  });

  inputData = stabilizedThis.map((el) => el[0]);

  // if (name) {
  //   inputData = inputData.filter(
  //     (order) =>
  //       order.orderNumber.toLowerCase().indexOf(name.toLowerCase()) !== -1 ||
  //       order.customer.name.toLowerCase().indexOf(name.toLowerCase()) !== -1 ||
  //       order.customer.email.toLowerCase().indexOf(name.toLowerCase()) !== -1
  //   );
  // }

  console.log('🚀 ~ file: common.ts:123 ~ type:', type);
  if (type !== 'All') {
    inputData = inputData.filter((order) => ORDER_TYPE_MAP[order.order_type] === type);
  }

  // if (!dateError) {
  //   if (startDate && endDate) {
  //     inputData = inputData.filter((order) => isBetween(order.createdAt, startDate, endDate));
  //   }
  // }

  return inputData;
}
