# Backend

## Overview

```
+-------------------------+
|       DAServerActor     |
|   (Manages Backends)    |
+-------------------------+
|
v
+-------------------------+
|       DABackend         | <- Trait for all backends
+-------------------------+
^
|
v
+---------------------------------------------------+
|               OpenDABackendManager                | <- Manages OpenDA-specific backends
| - Common OpenDA logic (batches, configs, etc.)    |
| - Reduces redundancy among OpenDA backends        |
+---------------------------------------------------+
|
v
+-------------------------------------+
|          OpenDAAdapter              | <- Trait for OpenDA-specific backend operations
| - submit_segment(), ...             |
| - Backend-specific operations       |
+-------------------------------------+
^
|
v
+-------------------+   +-------------------+
|  AvailAdapter     |   |   CelestiaAdapter | <- Actual backend-specific adapter implementations
+-------------------+   +-------------------+
```

DABackend provides a common interface for all backends.
The implementation of this trait is backend-specific, could be openDA or other backends.

## OpenDA

OpenDA abstracts various decentralized storage networks and cloud storage services as a single interface.
At the same time, it keeps the characteristics of each backend:

- Local/Cloud storage: high throughput, low latency. As buffer/cache layer bonded to a single sequencer(DA server) for
  responding most access.
- Decentralized storage: high availability. As final persistence layer.



