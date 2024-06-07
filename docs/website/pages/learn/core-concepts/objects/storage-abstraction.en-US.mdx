# Storage Abstraction

## Motivation

The biggest difference between smart contract programming languages and traditional programming languages is that smart contract programming languages need to provide standardized state storage interfaces within the language, shielding the underlying implementation of state storage. Smart contract applications primarily focus on their own business logic. 

**The objective of "Storage Abstraction" is to allow developers to define application state storage structures more flexibly in smart contracts, rather than being limited to standardized solutions provided by the platform.**

Let's first review the current solutions provided by smart contract platforms, and then introduce Rooch's "Storage Abstraction" solution.

### EVM Solution

In EVM, the state of a contract is stored through its field variables, and the virtual machine directly maps the contract's fields to persistent storage slots. The advantage of this approach is that it is almost transparent to developers, who can use storage as if it were memory, but there are also some drawbacks:

1. When designing complex state storage, scalability issues may arise, and developers still need to understand how contract variables are mapped to underlying storage.
2. Storage slots are not friendly to external access, and it is difficult for external systems to directly parse data in storage slots.
3. State is locked inside the contract, making it difficult for the chain to distinguish between user state and shared state within the contract. This poses challenges for [state fee](https://ethresear.ch/t/state-fees-formerly-state-rent-pre-eip-proposal-version-3/4996/4) and providing security guarantees for state at the chain level.
4. Similarly, because state is locked within the contract, contracts can only exchange "information", not "state".

### Move Solution

Move has made improvements to smart contract state storage. Applications need to perform explicit operations through global storage instructions. It mainly provides the following instructions:

1. `move_to<T:key>(&signer,T)`: Stores a resource of type `T` in the user state space of `signer`, which can only be executed by transactions initiated by the user.
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

![Storage Abstraction](/docs/rooch-design-storage-abstraction.svg)

Earlier, we mentioned that smart contracts need to provide state management interfaces for applications. We believe that the state management of smart contracts should be as simple as heap memory operations but with clear ownership. In Rooch, [Object](./object) is the basic unit of state storage. An Object is similar to a "smart pointer," holding an address of a state space while also representing ownership of that state space.

We then use Object and its dynamic fields to encapsulate upper-layer storage data structures, such as `Table`, `TypeTable`, and `Bag`. In Rooch, `Account` is also an Object, and the user's `Resource` is stored in the dynamic fields of `Object<Account>`. In this way, methods provided by the `account` module replace Move's global storage instructions, achieving a unified model of Object mode and `Account` Resource mode.

In Rooch, `ModuleStore` is a special Object that holds all the user-deployed `Package`. Each `Package` is a child Object of `ModuleStore`, and each `Package` contains multiple `Module`, with `Module` being the dynamic fields of the `Package` Object.

Based on this hierarchical object model, applications can design `AppSpecificStorage`. This allows the application's state to be stored within the same Object state space, while the state spaces of different applications can be distributed across different nodes. This forms the basis of the [DSTP](/learn/in-depth-tech/dstp) smart contract state layer.

### Design of State Tree

We believe that providing state proofs through a state tree is an important feature for interoperability between Web3 systems and external systems. Therefore, Rooch's state storage is designed around the state tree.

Rooch's state tree is implemented using a [Sparse Merkle Tree](https://github.com/rooch-network/rooch/tree/main/moveos/smt) which is very useful for the state tree due to its two key characteristics:

1. Performance is excellent because SMT's intermediate nodes are compressed and optimized.
2. SMT can simultaneously provide inclusion and no-inclusion proofs, especially useful in Rollup scenarios.


In Rooch, each Object represents an SMT (Sparse Merkle Tree), with the `state_root` field in the Object storing the root hash of the SMT. The Key of the dynamic field in an Object is the path of the SMT, and the Value is the leaf node of the SMT.

![statedb](/docs/rooch-design-statedb.svg)

In Rooch, the leaf nodes of the first layer of the state tree are Objects, and each Object carries a state subtree, which can store the dynamic fields of the Object or child Objects. For instance, BitcoinStore is an Object that stores all the states on the Bitcoin chain, with UTXO and Inscription as its child Objects.

This model can also be applied in applications. For example, in a game where the state is represented by Gameworld, all the game states are within this Object. This allows for parallel transactions and state partitioning between applications, with the Gameworld state existing on dedicated nodes.

However, all the subtrees will eventually be aggregated into a Root SMT. The root hash of this Root SMT will be written into a Bitcoin transaction, ensuring the verifiability of the state. Applications can also use Bitcoin's timestamp to prove that the user's state existed before a certain point in time.


#### Object

In Rooch's Object Storage, the data structure that is stored is `ObjectEntity<T>`, and `Object<T>` is like the handle or key to `ObjectEntity<T>`.

```move
module moveos_std::object{
    
    struct ObjectEntity<T>{
        id: ObjectID,
        owner: address,
        /// A flag to indicate whether the object is shared or frozen
        flag: u8,
        value: T,
    }

    struct Object<phantom T> has key, store {
        id: ObjectID,
    }
}
```

`ObjectEntity` does not have any `ability` and can only be accessed by the underlying storage API. Developers can access the data encapsulated in `ObjectEntity<T>` through `Object<T>`. The lifecycles of the two are the same. Developers only need to work with the `Object<T>` related API and do not need to care about `ObjectEntity<T>`.

For information on how to use Object, please refer to the [Object](../objects/object) documentation.


### Private Generics

Move's global storage instructions impose restrictions on its generic parameters to ensure the safety of contract states. In Rooch, we introduce the `#[private_generics]` annotation, allowing developers to attach these restrictions to custom functions. This enables developers to define richer storage data structures in smart contracts while ensuring the security of these structures.


## Summary

Rooch abstracts smart contract state storage by combining Move's Object model with state trees. This design not only inherits Move's clear state ownership and type safety features but also provides an efficient state proof mechanism through state trees, enhancing the system's verifiability and security.

In terms of the relationship between the execution platform, contract developers, and users, Rooch offers greater flexibility, allowing developers to freely design and implement application state storage structures. This enables developers to better meet the needs of various applications while ensuring the efficiency and security of storage, thereby promoting further development and innovation within the smart contract ecosystem.