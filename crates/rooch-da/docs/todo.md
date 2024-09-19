TODO
===

## Chunk Builder

Chunk Builder is a component to build chunks from batches, avoiding burst I/O to DA backend.

1. Persist batch into buffer(local persistence layer or other faster media) first, then return ok(if high-performance is
   preferred).
2. Split chunk into segments and submit segments to DA backend asynchronously.
3. Clean up batch buffer after segments being submitted to DA backend schedule.
