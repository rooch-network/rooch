import { ProductItemSkeleton } from './product-item-skeleton';

export const renderSkeleton = (
  <>
    {[...Array(12)].map((_, index) => (
      <ProductItemSkeleton key={index} />
    ))}
  </>
);
