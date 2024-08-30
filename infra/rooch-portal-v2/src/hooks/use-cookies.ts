import { useMemo, useState, useEffect, useCallback } from 'react';

import { isEqual } from 'src/utils/helper';

export type UseCookiesReturn<T> = {
  state: T;
  canReset: boolean;
  resetState: () => void;
  setState: (updateState: T | Partial<T>) => void;
  setField: (name: keyof T, updateValue: T[keyof T]) => void;
};

export function useCookies<T>(
  key: string,
  initialState: T,
  defaultValues: T,
  options?: {
    daysUntilExpiration?: number;
  }
): UseCookiesReturn<T> {
  const [state, set] = useState(initialState);

  const multiValue = initialState && typeof initialState === 'object';

  const canReset = !isEqual(state, defaultValues);

  useEffect(() => {
    const restoredValue: T = getStorage(key);

    if (restoredValue) {
      if (multiValue) {
        set((prevValue) => ({ ...prevValue, ...restoredValue }));
      } else {
        set(restoredValue);
      }
    }
  }, [key, multiValue]);

  const setState = useCallback(
    (updateState: T | Partial<T>) => {
      if (multiValue) {
        set((prevValue) => {
          setStorage<T>(key, { ...prevValue, ...updateState }, options?.daysUntilExpiration);
          return { ...prevValue, ...updateState };
        });
      } else {
        setStorage<T>(key, updateState as T, options?.daysUntilExpiration);
        set(updateState as T);
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [key, multiValue]
  );

  const setField = useCallback(
    (name: keyof T, updateValue: T[keyof T]) => {
      if (multiValue) {
        setState({ [name]: updateValue } as Partial<T>);
      }
    },
    [multiValue, setState]
  );

  const resetState = useCallback(() => {
    removeStorage(key);
    set(defaultValues);
  }, [defaultValues, key]);

  const memoizedValue = useMemo(
    () => ({
      state,
      setState,
      setField,
      resetState,
      canReset,
    }),
    [canReset, resetState, setField, setState, state]
  );

  return memoizedValue;
}

function getStorage(key: string) {
  try {
    const keyName = `${key}=`;

    const cDecoded = decodeURIComponent(document.cookie);

    const cArr = cDecoded.split('; ');

    let res;

    cArr.forEach((val) => {
      if (val.indexOf(keyName) === 0) res = val.substring(keyName.length);
    });

    if (res) {
      return JSON.parse(res);
    }
  } catch (error) {
    console.error('Error while getting from cookies:', error);
  }

  return null;
}

function setStorage<T>(key: string, value: T, daysUntilExpiration: number = 0) {
  try {
    const serializedValue = encodeURIComponent(JSON.stringify(value));
    let cookieOptions = `${key}=${serializedValue}; path=/`;

    if (daysUntilExpiration > 0) {
      const expirationDate = new Date(Date.now() + daysUntilExpiration * 24 * 60 * 60 * 1000);
      cookieOptions += `; expires=${expirationDate.toUTCString()}`;
    }

    document.cookie = cookieOptions;
  } catch (error) {
    console.error('Error while setting cookie:', error);
  }
}

function removeStorage(key: string) {
  try {
    document.cookie = `${key}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;`;
  } catch (error) {
    console.error('Error while removing cookie:', error);
  }
}
