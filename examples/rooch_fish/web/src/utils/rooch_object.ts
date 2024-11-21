// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export function transformObject(data: any): any {
  if (data === null || typeof data !== 'object') {
      return data; // 如果是基本类型，直接返回
  }

  if (data.decoded_value) {
    return transformObject(data.decoded_value);
  }

  if (Array.isArray(data)) {
      return data.map(item => transformObject(item)); // 处理数组
  }

  const newObject: { [key: string]: any } = {};
  for (const key in data) {
      if (data.hasOwnProperty(key)) {
          if (key === 'value' && typeof data[key] === 'object') {
              return transformObject(data[key]); // 直接返回 value 的内容
          } else if (key !== 'type') { // 只保留非 type 属性
              newObject[key] = transformObject(data[key]); // 递归处理对象属性
          }
      }
  }
  return newObject;
}
