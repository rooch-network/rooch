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
  totalPages: number
  onPageChange: (page: number) => void
}

const PaginationComponent: React.FC<PaginationComponentProps> = ({
  currentPage,
  totalPages,
  onPageChange,
}) => {
  return (
    <Pagination className="mt-6 justify-end">
      <PaginationContent>
        <PaginationItem>
          <PaginationPrevious
            onClick={() => currentPage > 1 && onPageChange(currentPage - 1)}
            className="cursor-pointer"
          />
        </PaginationItem>
        {Array.from({ length: totalPages }, (_, index) => (
          <PaginationItem key={index + 1}>
            <PaginationLink
              onClick={() => onPageChange(index + 1)}
              isActive={currentPage === index + 1}
              className="cursor-pointer"
            >
              {index + 1}
            </PaginationLink>
          </PaginationItem>
        ))}
        <PaginationItem>
          <PaginationNext
            onClick={() => currentPage < totalPages && onPageChange(currentPage + 1)}
            className="cursor-pointer"
          />
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  )
}

export default PaginationComponent
