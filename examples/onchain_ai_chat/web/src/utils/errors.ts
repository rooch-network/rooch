export function getErrorMessage(error: any): string {
  // Check if it's a Move abort error
  if (error?.message?.includes('Move abort')) {
    // Extract error code
    const errorCode = error.message.match(/Move abort: (\d+)/)?.[1];
    
    if (errorCode) {
      switch (errorCode) {
        case '2': // ErrorInsufficientBalance
          return 'Insufficient balance. Please deposit more RGas.';
        // Add more error codes as needed
        default:
          return `Transaction failed with error code: ${errorCode}`;
      }
    }
  }
  
  // Default error message
  return error?.message || 'An unexpected error occurred';
}