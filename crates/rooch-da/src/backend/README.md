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

