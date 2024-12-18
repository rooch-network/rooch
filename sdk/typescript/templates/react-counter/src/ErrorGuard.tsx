import { useSubscribeOnError } from '@roochnetwork/rooch-sdk-kit'
import { useEffect } from "react";

export function ErrorGuard() {
  const subscribeToError = useSubscribeOnError();

  useEffect(() => {
    const unsubscribe = subscribeToError((error) => { // session,
      console.error('Error occurred:', error);
    });

    return () => {
      unsubscribe();
    };
  }, [subscribeToError]);

  return <></>;
}
