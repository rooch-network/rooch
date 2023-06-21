# Storage Abstraction

## Motivation

The biggest difference between smart contract programming languages and traditional programming languages is that smart contract programming languages need to provide standardized state storage interfaces within the language, shielding the underlying implementation of state storage. Smart contract applications primarily focus on their own business logic. 

**The objective of "Storage Abstraction" is to allow developers to define their own state storage structures more flexibly in smart contracts, rather than being limited to standardized solutions provided by the platform.**

Let's first review the current solutions provided by smart contract platforms, and then introduce Rooch's "Storage Abstraction" solution.

### EVM Solution

In EVM, the state of a contract is stored through its field variables, and the virtual machine directly maps the contract's fields to persistent storage slots. The advantage of this approach is that it is almost transparent to developers, who can use storage as if it were memory, but there are also some drawbacks:

1. When designing complex state storage, scalability issues may arise, and developers still need to understand how contract variables are mapped to underlying storage.
2. Storage slots are not friendly to external access, and it is difficult for external systems to directly parse data in storage slots.
3. State is locked inside the contract, making it difficult for the chain to distinguish between user state and shared state within the contract. This poses challenges for [state fee](https://ethresear.ch/t/state-fees-formerly-state-rent-pre-eip-proposal-version-3/4996/4) and providing security guarantees for state at the chain level.
4. Similarly, because state is locked within the contract, contracts can only exchange "information", not "state".

### Move Solution

Move has made improvements to smart contract state storage. Applications need to perform explicit operations through global storage instructions. It mainly provides the following instructions:

1. `move_to<T:key>(signer)`: Stores a resource of type `T` in the user state space of `signer`, which can only be executed by transactions initiated by the user.
2. `move_from<T:key>(address):T`: Retrieves a resource of type `T` from the user state space.
3. `borrow_global<T:key>(address):&T`: Reads an immutable reference of type `T` from the user space.
4. `borrow_global_mut<T:key>(address):&mut T`: Reads a mutable reference of type `T` from the user space.

All of the above instructions include two security constraints:

1. The type `T` must be a struct with the `key` ability. The fields in the struct must be data types with the `store` ability.
2. The type `T` must be defined in the current module.

The first constraint ensures that only data types that developers explicitly declare as having the `store` ability can be written to the storage layer. The second constraint ensures the security of state between contracts. Storage operations of data structures in the current module can only be performed through methods provided by the current contract, which allows developers to encapsulate access verification logic in the methods to ensure security.

This approach has several advantages:

1. Ownership of states is explicit, making it easier for underlying systems to design state fees and provide security measures.
2. The type system is global, and contracts can exchange states with each other. Of course, this also depends on Move's type safety mechanism.

At the same time, Move provides the `Table<K,V>` extension, which allows developers to customize key-value storage.

### Sui Move Object

Sui Move has abandoned Move's global storage instructions and provided an Object model. Object is a special struct with the `key` ability, and its first field must be `UID`. Sui Move has designed a set of ownership mechanisms to define the ownership of Object. This design is mainly for achieving parallel transactions through a UTXO-like pattern. Clients therefore need to specify the Object that a contract will operate on in order to detect transaction conflicts quickly. Additionally, the model provides a parent-child relationship mechanism to facilitate the design of complex state structures by developers.

Security constraints of Sui Move's Object:

1. Objects have a clear owner (Owner) or are shared Objects. The virtual machine needs to verify the ownership of Objects when loading parameters.
2. There are two ways to transfer ownership of Objects inside Sui Move contracts. The first is through the `transfer<T:key>(T,address)` method, which can only be called in the module that defines `T`. This needs to be further encapsulated by developers for use by users, and developers can define customized verification logic for the transfer. The second is through the `public_transfer<T:key,store>(T,address)` method, which allows the Owner to directly transfer the Object. However, this method requires additional ability constraints for `T`, which must have the `store` ability.

Move's global storage instructions are based on users, and obtaining a resource requires knowing which user's space it belongs to and obtaining it based on the type of the resource. The Object model introduces an ID-based storage mechanism to Move's state storage. In addition, it provides users with the ability to directly operate on state by bypassing developers through `public_transfer`.

### Rooch Storage Abstraction Design Principles

After analyzing the above solutions, we find that the storage model of a contract mainly defines the relationship between the execution platform of the contract, developers, and users. Therefore, in Rooch, we propose the concept of "Storage Abstraction" by drawing on the advantages of the above solutions. The design principles are as follows:

1. Simple abstraction principle. The underlying storage interface should be abstract and simple as much as possible.
2. Contract-oriented principle. More rich state storage interfaces should be implemented through contracts, rather than relying on the implementation of the underlying smart contract platform.
3. Self-awareness principle. The contract is aware of the storage data structure used in the contract.
4. Interoperability-friendly principle. External systems can easily access the data structure defined by the contract, while also being able to obtain storage proofs easily.
5. Clear ownership principle. The ownership of all storage data is clear, and the relationship between the execution platform of the contract, developers, and users is clear.

## Design Solution

![Storage Abstraction](../static/design/rooch-design-storage-abstraction.svg)

1. `RawTable` provides the lowest-level Key Value storage interface, where all contract state changes are ultimately uniformly changed as the Key Value change set of `RawTable`.
2. `Table<K,V>` is implemented based on `RawTable`, constraining the types of Key and Value.
3. `TypeTable` is implemented based on `RawTable`, using the type of Value as the Key for the storage structure.
4. `ObjectStorage` is implemented based on `RawTable`, providing storage capabilities for Objects.
5. `ResourceTable` and `ModuleTable` are implemented based on `TypeTable` and `Table` respectively, encapsulating them into `AccountStorage`, providing storage interfaces for Move's user space storage to replace Move's global storage instructions. At the same time, `AccountStorage` also provides an interface for operating Modules in the contract, which is convenient for defining the upgrade logic of the contract in the future.
6. Developers can encapsulate their own application-specific storage interfaces based on the above storage structures.

### Design of State Tree

We believe that providing state proofs through a state tree is an important feature for interoperability between Web3 systems and external systems. Therefore, Rooch's state storage is designed around the state tree.

Rooch's state tree is implemented using a [Sparse Merkle Tree](https://github.com/rooch-network/smt) which is very useful for the state tree due to its two key characteristics:

1. Performance is excellent because SMT's intermediate nodes are compressed and optimized.
2. SMT can simultaneously provide inclusion and no-inclusion proofs, especially useful in Rollup scenarios.

The overall architecture of Rooch's StateDB state tree is shown in the following diagram:

![statedb](../static/design/rooch-design-statedb.svg)

The first layer of the state tree in Rooch is an SMT with ObjectID as the key and the serialized binary data of the Object as the value. It can be understood as a global ObjectStore. There are three special Objects in it:

1. AccountStorage Object is saved with the user's Address as the key, and will be described in detail later.
2. Resource Table Object is saved with Hash(Address + 0) as the key. The ResourceTable's key is the resource type, and the value is the Resource.
3. Module Table Object is saved with Hash(Address + 1) as the key. The ModuleTable's key is the Module Name, and the value is the Module bytecode.

In Rooch, each Table in Move corresponds to an SMT, and the Table Object mainly stores the root of the SMT. When loading data from a Table, the root of the SMT is first obtained through the Table Object ID, and the SMT is then initialized to obtain the value through the key. Tables can also be fields in Struct, with the serialized Table ID (TableHandle) stored in the Struct. In this way, Rooch's state tree is a hierarchy of nested SMTs.


### StorageContext

The StorageContext contains two fields: `TxContext`, which contains information related to the current transaction, and `ObjectStore`, which corresponds to the first layer of the state tree.

```move
module moveos_std::storage_context{
    struct StorageContext {
        tx_context: TxContext,
        object_storage: ObjectStorage,
    }
}
```

Developers can define the StorageContext parameter in the `entry` method, and the MoveVM will automatically fill in the parameter.

```move
module example::my_module{
    public entry fun my_entry_fun(ctx: &mut StorageContext){
        //function logic
    }
}
```

### RawTable

In Rooch, a RawTable extension has been implemented for Move, which is an extension of the original Table and removes the constraints on Key and Value.

The following are some of the key interfaces provided by RawTable in Move:

```move
module moveos_std::raw_table{
    public(friend) fun add<K: copy + drop, V>(table_handle: &ObjectID, key: K, val: V);

    public(friend) fun borrow<K: copy + drop, V>(table_handle: &ObjectID, key: K);

    public(friend) fun borrow_mut<K: copy + drop, V>(table_handle: &ObjectID, key: K): &mut V;

    public(friend) fun remove<K: copy + drop, V>(table_handle: &ObjectID, key: K): V;

    public(friend) fun contains<K: copy + drop, V>(table_handle: &ObjectID, key: K): bool;
}
```

TableHandle is the ObjectID of the TableObject. The upper-level data structure of RawTable can directly operate on any type of K, V, like a raw-level KV storage interface. Each RowTable corresponds to a SMT in the StateDB. The ObjectID of the global ObjectStore is a special ObjectID with all bytes set to `0`.

Currently, all methods in RawTable are `friend` methods, and developers cannot directly operate on them. They need to operate through the upper-level data structures that encapsulate RawTable, such as `Table<K,V>`, `TypeTable`, and `ObjectStorage`.

### Rooch Object

Objects in Rooch adopt a Box mode where they are equivalent to a unique ID box that can encapsulate a type `T` to create an Object.

```move
module moveos_std::object{
    struct Object<T: key>{
        id: ObjectID,
        owner: address,
        value: T,
    }
}
```

Rooch Objects are also a use case for the [Hot Potato](https://examples.sui.io/patterns/hot-potato.html) pattern in Move. Objects do not have any `ability`, so they cannot be `drop`, `copy`, or `store`, and can only be handled by ObjectStorage API after creation.

The advantage of this design is that no additional requirements need to be placed on the definition of Move's Struct, nor is additional support of Native methods required, since the concept of Object can be defined directly in Move. ObjectStorage is a storage space outside of a user's storage space, with a unique global ID as the key, to address situations in which global storage is needed.

### ObjectStorage

ObjectStorage provides interfaces related to global storage of Objects. It is implemented based on RawTable and is defined as follows:

```move
module moveos_std::object_storage{
    #[private_generics(T)]
    /// Borrow Object from object store with object_id
    public fun borrow<T: key>(this: &ObjectStorage, object_id: ObjectID): &Object<T>;

    #[private_generics(T)]
    /// Borrow mut Object from object store with object_id
    public fun borrow_mut<T: key>(this: &mut ObjectStorage, object_id: ObjectID): &mut Object<T>;

    #[private_generics(T)]
    /// Remove object from object store
    public fun remove<T: key>(this: &mut ObjectStorage, object_id: ObjectID): Object<T>;

    #[private_generics(T)]
    /// Add object to object store
    public fun add<T: key>(this: &mut ObjectStorage, obj: Object<T>);

    #[private_generics(T)]
    public fun contains<T: key>(this: &ObjectStorage, object_id: ObjectID): bool;
}
```

The above methods are ensured to only be called directly by Modules containing `T` through [private_generics](https://github.com/rooch-network/rooch/issues/64), and the safety model follow to the constraints of the Move global storage instructions.

>TBD: Whether to provide a method that allows Owners to directly operate on Objects, similar to Sui's `public_transfer`, needs further research to decide.

### TypeTable

TypeTable is a special type of Table that simulates Move's global storage instructions, with the type as the key and an instance of the type as the value. Implemented based on RawTable, it provides the following API:

```move
module moveos_std::type_table {

    #[private_generics(V)]
    public fun add<V: key>(table: &mut TypeTable, val: V);

    #[private_generics(V)]
    public fun borrow<V: key>(table: &TypeTable): &V;

    #[private_generics(V)]
    public fun borrow_mut<V: key>(table: &mut TypeTable): &mut V;

    #[private_generics(V)]
    public fun remove<V: key>(table: &mut TypeTable): V;

    #[private_generics(V)]
    public fun contains<V: key>(table: &TypeTable): bool;
}
```

The methods in TypeTable rely on the type safety guarantee of `private_generics`, with constraints similar to Move's global storage instructions.

### AccountStorage

AccountStorage is an abstraction of a user's storage space in Move, which contains two Tables, the Resource Table and Module Table, allowing Resources and Modules to be directly manipulated in Move without the need for global storage instructions.

It mainly provides the following API:

```move
module moveos_std::account_storage{
    
    struct AccountStorage has key {
        resources: TypeTable,
        modules: Table<String, MoveModule>,
    }

    #[private_generics(T)]
    /// Borrow a resource from the account's storage
    /// This function equates to `borrow_global<T>(address)` instruction in Move
    public fun global_borrow<T: key>(ctx: &StorageContext, account: address): &T;

    #[private_generics(T)]
    /// Borrow a mut resource from the account's storage
    /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
    public fun global_borrow_mut<T: key>(ctx: &mut StorageContext, account: address): &mut T;

    #[private_generics(T)]
    /// Move a resource to the account's storage
    /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
    public fun global_move_to<T: key>(ctx: &mut StorageContext, account: &signer, resource: T);

    #[private_generics(T)]
    /// Move a resource from the account's storage
    /// This function equates to `move_from<T>(address)` instruction in Move
    public fun global_move_from<T: key>(ctx: &mut StorageContext, account: address): T;

    #[private_generics(T)]
    /// Check if the account has a resource of the given type
    /// This function equates to `exists<T>(address)` instruction in Move
    public fun global_exists<T: key>(ctx: &StorageContext, account: address) : bool;

    /// Check if the account has a module with the given name
    public fun exists_module(ctx: &StorageContext, account: address, name: String): bool;

    /// Publish modules to the account's storage
    public fun publish_modules(ctx: &mut StorageContext, account: &signer, modules: vector<MoveModule>);
}
```

The above methods can replace the global storage instructions in Move, so that all storage-related operations in Move are ultimately unified to Operations on Tables. 

At the same time,this approach also solves a reference problem in Move whereby developers cannot return a reference borrowed through the `borrow_global<T>` instruction in their methods, since this reference is created out of thin air and cannot be returned as a method return value. However, if the reference is borrowed through `StorageContext`, then that goal can be achieved, and the lifetime of the reference is bound to the lifetime of `StorageContext`. 

In addition, AccountStorage also provides module-related methods, making it easy to deploy Move contracts in Move and allowing developers to use contracts to define contract deployment rules, such as upgrading the contract through DAO governance.

> TBD: Whether to completely abandon the global storage instructions in Move, or to simultaneously provide both methods of operation, needs further research to decide.

### Unified State AccessPath API

Since Rooch's StateDB is a nested SMT, we can provide a unified [access path API](https://github.com/rooch-network/rooch/issues/58).

`/table/$table_handle/$key1,$key2`: Accesses the data of `$key1,$key2` in the Table of `$table_handle`. If `$table_handle` is `0x0`, it means that the accessed data is from the first level of the SMT.

In addition, the following aliases are provided for accessing data:

* `/object/$object_id`: A shortcut for accessing the first layer of the SMT, which is equivalent to `/table/0x0/$object_id`.
* `/module/$address/$module_name`: Accesses a Module in the AccountStorage of a user, which is equivalent to `/table/NamedTable($address,resource)/hex($module_name)`.
* `/resource/$address/$resource_struct_tag`: Accesses a Resource in the AccountStorage of a user, which is equivalent to `/table/NamedTable($address,module)/hex($resource_struct_tag)`.

Since SMT stores serialized values of Move's Struct, external systems can directly deserialize them into JSON or other data structures in programming languages, which is friendly to developers.

## Summary

Rooch abstracts application storage needs into a KV storage interface, unifying the way application state is handled while simplifying the storage logic in the VM layer and providing the ability to extend new storage spaces. Based on the KV interface, richer and safer storage space interfaces are encapsulated in Rooch contracts to meet the storage needs of different application scenarios.

Rooch's state tree and unified access path API facilitate combinations with external systems and ensure interoperability.

Finally, regarding the relationship between the execution platform, contract developers, and users, Rooch continues the security constraints of Move, with clear ownership of the state, but operations on the state need to be constrained by API defined by developers. Also, Rooch is exploring solutions that allow users to directly operate on their own state.

`Storage Abstraction` is a new concept, and as application scenarios become more diverse, new storage needs will emerge. Future explorations could include:

1. Providing richer storage solutions tailored to specific application scenarios.
2. Extending the abstraction of SMT, whereby any storage structure that can provide a state proof can theoretically be mapped in Move as a Table, such as the state tree of various chains, or Git in Offchain.
