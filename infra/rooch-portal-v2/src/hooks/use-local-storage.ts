import { useMemo, useState, useEffect, useCallback } from 'react';

import { isEqual } from 'src/utils/helper';
import { localStorageGetItem } from 'src/utils/storage-available';

export type UseLocalStorageReturn<T> = {
  state: T;
  canReset: boolean;
  resetState: () => void;
  setState: (updateState: T | Partial<T>) => void;
  setField: (name: keyof T, updateValue: T[keyof T]) => void;
};

export function useLocalStorage<T>(key: string, initialState: T): UseLocalStorageReturn<T> {
  const [state, set] = useState(initialState);

  const multiValue = initialState && typeof initialState === 'object';

  const canReset = !isEqual(state, initialState);

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
          setStorage<T>(key, { ...prevValue, ...updateState });
          return { ...prevValue, ...updateState };
        });
      } else {
        setStorage<T>(key, updateState as T);
        set(updateState as T);
      }
    },
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
    set(initialState);
    removeStorage(key);
  }, [initialState, key]);

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

export function getStorage(key: string) {
  try {
    const result = localStorageGetItem(key);

    if (result) {
      return JSON.parse(result);
    }
  } catch (error) {
    console.error('Error while getting from storage:', error);
  }

  return null;
}

export function setStorage<T>(key: string, value: T) {
  try {
    const serializedValue = JSON.stringify(value);
    window.localStorage.setItem(key, serializedValue);
  } catch (error) {
    console.error('Error while setting storage:', error);
  }
}

export function removeStorage(key: string) {
  try {
    window.localStorage.removeItem(key);
  } catch (error) {
    console.error('Error while removing from storage:', error);
  }
}
