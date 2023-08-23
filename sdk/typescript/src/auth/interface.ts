import { Bytes } from '../types'

export interface IAuthorization {
  scheme: number
  payload: Bytes
}

export interface IAuthorizer {
  auth(callData: Bytes): IAuthorization
}
