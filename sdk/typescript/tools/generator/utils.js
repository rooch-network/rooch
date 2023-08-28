// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export function getType(schema, schemas, alias) {
  if (typeof schema === "string" || typeof schema === "boolean") {
      return alias ? alias(schema) : schema;
  }

  if ('anyOf' in schema) {
      // Generate a union type from the anyOf array
      return schema.anyOf.map(subSchema => {
          if (subSchema.type === 'null') {
              return 'null';
          } else {
              return getType(subSchema, schemas, alias);
          }
      }).join(' | ');
  }

  if ('oneOf' in schema) {
      // Generate a union type from the oneOf array
      return schema.oneOf.map(subSchema => getType(subSchema, schemas, alias)).join(' | ');
  }

  if ('type' in schema) {
      if (schema.type === 'integer') {
          return 'number';  // Convert to TypeScript type
      } else if (schema.type === 'object') {
          if (schema.additionalProperties) {
              // This is an object with dynamic keys
              const valueType = getType(schema.additionalProperties, schemas, alias);
              return `{ [key: string]: ${valueType} }`;
          } else {
              // This is an object with a fixed set of properties
              const properties = schema.properties;
              const propertyTypes = Object.keys(properties).map(key => {
                  const propertySchema = properties[key];
                  const propertyType = getType(propertySchema, schemas, alias);
                  return `${key}: ${propertyType}`;
              });
              return `{ ${propertyTypes.join(', ')} }`;
          }
      } if (schema.type === 'array') {
          // This is an array type
          const itemType = getType(schema.items, schemas, alias);
          return `${itemType}[]`;
      } else if (Array.isArray(schema.type)) {
          return schema.type.map(t => {
              if (t === 'null') {
                  return 'null';
              } else if (t === 'integer') {
                  return 'number';
              } else if (t === 'array') {
                  const itemType = getType(schema.items, schemas, alias);
                  return `${itemType}[]`;
              } else {
                  return alias ? alias(t) : t;
              }
          }).join(' | ');
      } else {
          return alias ? alias(schema.type) : schema.type;
      }
  } else if ('$ref' in schema) {
      const originalRefName = schema.$ref.split('/').pop();  // Extract the $ref name
      let refName = originalRefName;
      if (refName.indexOf('::') !== -1) {
          refName = refName.replace(/::/g, "_").replace(/<u8>/g, "_U8Array"); // When $ref contains ::, we take the last part
      }
      if (originalRefName in schemas) {
          if (originalRefName === 'alloc::vec::Vec<u8>') {
              return 'Uint8Array';
          }
          // Add this condition to handle the case when the referred schema is also a $ref
          else if ('$ref' in schemas[originalRefName]) {
              return getType(schemas[originalRefName], schemas, alias);
          }
          else {
              return alias ? alias(refName) : refName;  // Reference other schema
          }
      } else {
          throw new Error(`Reference ${originalRefName} not found in schemas`);
      }
  } else {
      return 'unknown';
  }
}
