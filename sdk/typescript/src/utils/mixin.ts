// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/**
In TypeScript, we can’t inherit or extend from more than one class,
Mixins helps us to get around that by creating a partial classes 
that we can combine to form a single class that contains all the methods and properties from the partial classes.
{@link https://www.typescriptlang.org/docs/handbook/mixins.html#alternative-pattern}
*/
export function applyMixin(targetClass: any, baseClass: any, baseClassProp: string) {
  // Mixin instance methods
  Object.getOwnPropertyNames(baseClass.prototype).forEach((propertyName) => {
    const propertyDescriptor = Object.getOwnPropertyDescriptor(baseClass.prototype, propertyName);
    if (!propertyDescriptor) return;
    // eslint-disable-next-line func-names
    propertyDescriptor.value = function (...args: any) {
      return (this as any)[baseClassProp][propertyName](...args);
    };
    Object.defineProperty(targetClass.prototype, propertyName, propertyDescriptor);
  });
  // Mixin static methods
  Object.getOwnPropertyNames(baseClass).forEach((propertyName) => {
    const propertyDescriptor = Object.getOwnPropertyDescriptor(baseClass, propertyName);
    if (!propertyDescriptor) return;
    // eslint-disable-next-line func-names
    propertyDescriptor.value = function (...args: any) {
      return (this as any)[baseClassProp][propertyName](...args);
    };
    if (targetClass.hasOwnProperty.call(targetClass, propertyName)) {
      // The mixin has already been applied, so skip applying it again
      return;
    }
    Object.defineProperty(targetClass, propertyName, propertyDescriptor);
  });
}