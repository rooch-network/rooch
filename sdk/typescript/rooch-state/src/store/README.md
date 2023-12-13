# Store

# DATA MODEL

Each piece of data in Store is stored as a record in a table. We can think of tables in two ways, either as a relational database or as a key-value store.
Each table is identified by a unique ResourceId tableId.
Each record in a table is identified by a unique bytes32[] keyTuple. You can think of the key tuple as a composite key in a relational database, or as a nested mapping in a key-value store.
Each table has a ValueSchema that defines the types of data stored in the table. You can think of the value schema as the column types in a table in a relational database, or the type of structs stored in a key-value store.

# READING AND WRITING DATA

The StoreCore library implements low-level methods for reading and writing data in a Store contract and the IStore interface exposes some of these methods to external callers.
Note that when exposing IStoreWrite methods to external callers, it is necessary to implement an access control mechanism, or use the access control mechanisms provided by World.
