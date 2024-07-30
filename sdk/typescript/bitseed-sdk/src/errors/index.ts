export class BitseedSDKError extends Error {
  constructor(message: string) {
    super(message)
    this.name = "BitseedSDKError"
  }
}
