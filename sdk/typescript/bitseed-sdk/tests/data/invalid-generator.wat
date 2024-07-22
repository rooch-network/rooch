(module
  ;; Import the JavaScript console.log function.
  (import "console" "log" (func $log (param i32)))

  ;; Memory declaration with one page (64KiB)
  (memory (export "memory") 1)

  ;; Data segment to store the "Hello, World!" string in linear memory.
  (data (i32.const 0) "Hello, World!\00")

  ;; Exported function that calls console.log with the pointer to the string.
  (func (export "helloWorld")
    i32.const 0  ;; The pointer to the "Hello, World!" string in linear memory.
    call $log    ;; Call the imported console.log function.
  )
)
