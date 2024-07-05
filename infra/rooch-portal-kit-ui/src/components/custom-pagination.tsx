// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React from 'react'
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from '@/components/ui/pagination.tsx'

interface PaginationComponentProps {
  currentPage: number
  hasNextPage: boolean
  onPageChange: (page: number) => void
}

const PaginationComponent: React.FC<PaginationComponentProps> = ({
  currentPage,
  hasNextPage,
  onPageChange,
}) => {
  return (
    <Pagination className="mt-6 justify-end">
      <PaginationContent>
        <PaginationItem>
          <PaginationPrevious
            href="#"
            onClick={() => currentPage > 0 && onPageChange(currentPage - 1)}
            className={`cursor-pointer border-none hover:bg-inherit ${
              currentPage <= 0 ? 'text-gray-500 cursor-not-allowed hover:text-gray-500' : ''
            }`}
            isActive={currentPage <= 0}
          />
        </PaginationItem>
        <PaginationItem>
          <PaginationLink
            href="#"
            onClick={() => onPageChange(currentPage)}
            isActive={true}
            className="cursor-pointer"
          >
            {currentPage + 1}
          </PaginationLink>
        </PaginationItem>
        <PaginationItem>
          <PaginationNext
            href="#"
            onClick={() => currentPage > 0 && onPageChange(currentPage + 1)}
            className={`cursor-pointer border-none hover:bg-inherit ${
              !hasNextPage ? 'text-gray-500 cursor-not-allowed hover:text-gray-500' : ''
            }`}
            isActive={hasNextPage}
          />
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  )
}

export default PaginationComponent
