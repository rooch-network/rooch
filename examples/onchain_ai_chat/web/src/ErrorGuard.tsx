import { useSubscribeOnError } from '@roochnetwork/rooch-sdk-kit'
import { useEffect } from "react";
import toast from 'react-hot-toast';

export function ErrorGuard() {
  const subscribeToError = useSubscribeOnError();

  useEffect(() => {
    const unsubscribe = subscribeToError((error) => {
      // Display error toast in bottom-left corner
      toast.error(error.message, {
        position: 'bottom-left',
        duration: 5000, // Auto dismiss after 5 seconds
      });

      // Keep console log for debugging
      console.error('Error occurred:', error);
    });

    return () => {
      unsubscribe();
    };
  }, [subscribeToError]);

  // Add toast container to render notifications
  return (
    <>
      <div className="fixed bottom-4 left-4 z-50" /> {/* Toast anchor point */}
    </>
  );
}
