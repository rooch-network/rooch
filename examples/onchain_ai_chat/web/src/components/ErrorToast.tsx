import { useState, useEffect } from 'react';
import { XMarkIcon } from '@heroicons/react/24/outline';

interface ErrorToastProps {
  message: string;
  onClose: () => void;
  autoHideDuration?: number;
}

export function ErrorToast({ message, onClose, autoHideDuration = 5000 }: ErrorToastProps) {
  useEffect(() => {
    const timer = setTimeout(onClose, autoHideDuration);
    return () => clearTimeout(timer);
  }, [onClose, autoHideDuration]);

  return (
    <div className="fixed bottom-20 right-4 flex items-center gap-2 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg shadow-lg">
      <span className="text-sm">{message}</span>
      <button
        onClick={onClose}
        className="text-red-500 hover:text-red-700"
      >
        <XMarkIcon className="h-5 w-5" />
      </button>
    </div>
  );
}