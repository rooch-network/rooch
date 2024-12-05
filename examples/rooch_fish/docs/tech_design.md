# RoochFish Contract Technical Solution (Optimized Version)

## 1. Introduction

### 1.1 Project Background

"RoochFish" is a multiplayer online game based on the Rooch blockchain platform. Players compete in multiple dynamic fish ponds by purchasing and controlling virtual fish. The game combines growth, strategy, and economic elements to provide players with an interesting and economically motivated gaming experience. The game introduces a physical system and a simplified food source tracking mechanism, which increases the strategic depth and game balance.

### 1.2 Purpose of the Technical Solution

This technical solution aims to describe the contract implementation of the RoochFish game in detail, including demand analysis, system design, and detailed implementation solutions. The solution will guide the development team to complete contract development efficiently and safely.

## 2. Requirements Overview

### 2.1 Functional Requirements

- **Purchase and Generation of Fish**: Players use RGAS tokens to purchase fish, and the fish are randomly generated in the fish pond with an initial size and physical value.
- **Fish movement and stamina**: Players control the movement of fish, which consumes stamina and automatically recovers stamina every second. Fish cannot move when stamina is 0.
- **Fish growth**: Fish increase in size by eating fish or food smaller than themselves, and recover stamina immediately when eating food or small fish.
- **Fish destruction and rewards**: Fish can be moved to the exit location for destruction, and players receive RGAS token rewards based on the size of the fish. Fish will "die" when they reach their maximum size, and produce food.
- **Feeding food**: Players use RGAS tokens to feed food to the fish pond, and food is randomly generated in the fish pond.
- **Food source tracking**: Food records the player address of its source for reward distribution.
- **Reward distribution**: When a fish is destroyed or dies, rewards are distributed to the developer, the owner of the fish, and the player who is the source of the food according to a certain ratio.
- **New fish protection mechanism**: New fish cannot be eaten by other fish within 1 minute.

### 2.2 Non-functional requirements

- **Security**: The contract needs to prevent cheating and attacks to ensure the fairness of the game.

- **Performance**: Optimize the efficiency of contract execution and reduce Gas consumption.

- **Scalability**: The code design needs to be easy to expand and support future function additions.

- **User experience**: Simplify the interaction process to ensure timely feedback and good experience.

## 3. System analysis

### 3.1 Role analysis

- **Player**: Buy and control fish, feed food, and get rewards.

- **Fish**: The main entity in the game, with attributes such as size, physical strength, and position.

- **Food**: Items that fish can eat, increase the size of fish and restore physical strength.

- **Fish pond**: The main scene of the game, accommodating fish and food.

### 3.2 Key entities and attributes

#### Fish

- **id**: unique identifier
- **owner**: owner address
- **size**: size
- **position**: position (x, y)
- **stamina**: stamina value (0-10)
- **last_stamina_update**: timestamp of last stamina recovery
- **created_at**: creation timestamp
- **food_sources**: food source record (list)

#### Food

- **id**: unique identifier
- **size**: size
- **value**: value
- **position**: position (x, y)
- **feeder**: player address of food source

#### Player

- **address**: player address
- **feed_amount**: cumulative amount of food fed
- **reward**: cumulative reward

#### Fish Pond

- **fishes**: List of current fish

- **foods**: List of current food

- **max_fish_size**: Maximum size limit of fish

## 4. System Design

### 4.1 Module Structure

- **RoochFish Main Module**: Main logic and entry function of the game.

- **Fish Module**: Defines the structure and behavior of fish.

- **Food Module**: Defines the structure and behavior of food.

- **Pond Module**: Manages the state of the fish pond.

- **Player Module**: Manages the player's feeding and reward records.

- **Utils Tool Module**: Provides auxiliary functions, such as random number generation, time acquisition, etc.

### 4.2 Data structure design #### Fish module ```move struct Fish has key, store { id: u64, owner: address, size: u64, position: (u64, u64), stamina: u8, last_stamina_update: u64, created_at: u64, food_sources: vector<(address, u64)>, } ``` #### Food module ```move struct Food has key, store { id: u64, size: u64, value: u64, position: (u64, u64), feeder: address, } ``` #### Player module ```move struct Player has key, store { address: address, feed_amount: u64, reward: u64,
}
```

#### Pond module

```move
struct PondState has key, store {
fishes: vector<Object<Fish>>,
foods: vector<Object<Food>>,
max_fish_size: u64,
}
```

### 4.3 Key algorithms and logic

#### 4.3.1 Physical system

- **Automatic recovery**: Fish recover 1 physical point per second, up to 10 points.

- **Movement consumption**: Each move consumes 1 physical point, and insufficient physical strength cannot move.

- **Instant recovery**: Eat food or small fish, and physical strength will be restored to 10 points immediately.

#### 4.3.2 Movement logic

- **Boundary detection**: Ensure that the movement of fish does not exceed the scope of the fish pond.

- **Collision detection**: Detect collision with other fish or food.

#### 4.3.3 Eating fish and eating food

- **Eating fish conditions**: You can only eat fish smaller than yourself, and the target fish is not in the protection period.

- **Eating food**: Increase the size of the fish and restore physical strength.

- **Food source record**: Record the feeder address and value of the food for reward distribution.

#### 4.3.4 Reward distribution

- **When the fish is destroyed**:
- 1% reward to the developer.
- 79% reward to the owner of the fish.
- 20% is distributed to the feeder according to the contribution ratio based on the food source.
- **When the fish dies**:
- 1% reward to the developer.
- 20% is distributed to the feeder according to the contribution ratio based on the food source.
- 79% is converted into food, which brings benefits to the owner of the fish when eaten by other fish.

## 5. Module Detailed Design

### 5.1 RoochFish Main Module

```move
module <ADDR>::RoochFish {
public entry fun purchase_fish(ctx: &mut TxContext);
public entry fun move_fish(fish_id: u64, direction: u8, ctx: &mut TxContext);
public entry fun feed_food(amount: u64, ctx: &mut TxContext);
public entry fun destroy_fish(fish_id: u64, ctx: &mut TxContext);
}
```

#### Function Description

- **purchase_fish**: The player purchases fish, generates new fish and adds them to the fish pond.
- **move_fish**: The player controls the movement of the fish, handles physical strength and collision logic.
- **feed_food**: The player feeds food, generates food and adds it to the fish pond.
- **destroy_fish**: Player destroys fish, collects rewards and handles distribution.

### 5.2 Fish module

```move
module <ADDR>::Fish {
struct Fish has key, store { ... }

public fun new(owner: address, id: u64, ctx: &mut TxContext): Object<Fish>;

public fun move(fish: &mut Fish, direction: u8, ctx: &mut TxContext);

public fun grow(fish: &mut Fish, amount: u64, feeder: address);

public fun is_protected(fish: &Fish, current_time: u64): bool;

public fun auto_recover_stamina(fish: &mut Fish, current_time: u64);
}
```

#### Function description

- **new**: Create a new fish and initialize attributes.

- **move**: Handle the movement of fish and consume stamina.
- **grow**: The fish eats food or small fish, increases in size and recovers stamina, and records the source of food.
- **is_protected**: Determines whether the fish is in the protection period.
- **auto_recover_stamina**: Automatically recovers stamina.

### 5.3 Food module

```move
module <ADDR>::Food {
struct Food has key, store { ... }

public fun new(id: u64, feeder: address, size: u64, value: u64, ctx: &mut TxContext): Object<Food>;
}
```

#### Function description

- **new**: Create new food, record the source and value.

### 5.4 Pond module

```move
module <ADDR>::Pond {
struct PondState has key, store { ... }

public fun add_fish(fish: Object<Fish>);

public fun remove_fish(fish_id: u64);

public fun add_food(food: Object<Food>);

public fun remove_food(food_id: u64);

public fun get_state(): &mut PondState;
}
```

#### Function description

- **add_fish**: Add fish to the pond.

- **remove_fish**: Remove fish from the pond.

- **add_food**: Add food to the pond.

- **remove_food**: Remove food from the pond.

- **get_state**: Get the current state of the pond.

### 5.5 Player module

```move
module <ADDR>::Player {
struct Player has key, store { ... }

public fun add_feed(address: address, amount: u64);

public fun add_reward(address: address, amount: u64);

public fun get_state(address: address): &mut Player;
}
```

#### Function description

- **add_feed**: Record the player's feed amount.

- **add_reward**: Add the player's reward.
- **get_state**: Get the player's status information.

### 5.6 Utils Tool Module

```move
module <ADDR>::Utils {
public fun random_position(): (u64, u64);

public fun current_timestamp(): u64;
}
```

#### Function Description

- **random_position**: Generates random position coordinates.

- **current_timestamp**: Gets the current timestamp.

## 6. Implementation details

### 6.1 Purchase fish

```move
public entry fun purchase_fish(ctx: &mut TxContext) {
let sender = ctx.sender();
// Deduct RGAS tokens
coin::burn<RGAS>(sender, purchase_amount);

// Create a new fish
let fish_id = generate_unique_id();
let position = Utils::random_position();
let created_at = Utils::current_timestamp();
let fish = Fish::new(sender, fish_id, ctx);

// Add to the fish pond
Pond::add_fish(fish);
}
```

### 6.2 Move fish

```move
public entry fun move_fish(fish_id: u64, direction: u8, ctx: &mut TxContext) {
let current_time = Utils::current_timestamp();
let fish = Pond::get_fish_mut(fish_id);
// Verify ownership
assert!(fish.owner == ctx.sender(), ErrorFishNotOwned);
// Automatically recover stamina
Fish::auto_recover_stamina(&mut fish, current_time);
// Check stamina
assert!(fish.stamina >= 1, ErrorInsufficientStamina);
// Consume stamina
fish.stamina -= 1;
// Move fish
Fish::move(&mut fish, direction, ctx);
// Handle collisions
handle_collisions(&mut fish, ctx);
// Check if fish is full
if fish.size >= Pond::get_state().max_fish_size {
handle_overgrown_fish(&fish, ctx);
}
}
```

### 6.3 Feeding food

```move
public entry fun feed_food(amount: u64, ctx: &mut TxContext) {
let feeder = ctx.sender();
// Deduct RGAS tokens
coin::burn<RGAS>(feeder, amount);

// Record feeding
Player::add_feed(feeder, amount);

// Generate food
for _ in 0..amount {
let food_id = generate_unique_id();
let food = Food::new(food_id, feeder, 1, 1, ctx); // Assume that the size and value of each unit of food is 1
Pond::add_food(food);
}
}
```

### 6.4 Destroying fish

```move
public entry fun destroy_fish(fish_id: u64, ctx: &mut TxContext) {
let fish = Pond::get_fish(fish_id);

// Verify ownership and position
assert!(fish.owner == ctx.sender(), ErrorFishNotOwned);
assert!(is_at_exit(fish.position), ErrorNotAtExit);

// Calculate rewards
let total_reward = calculate_reward(fish.size);

// Distribute rewards
distribute_rewards(&fish, total_reward);

// Remove fish
Pond::remove_fish(fish_id);
}
```

### 6.5 Dealing with overgrown fish

```move
fun handle_overgrown_fish(fish: &Fish::Fish, ctx: &mut TxContext) {
let fish_size = fish.size;
let total_value = calculate_reward(fish_size);

// Distribute rewards
distribute_rewards(fish, total_value);

// Generate food
let remaining_value = total_value * 79 / 100; let food_value = remaining_value / 10; let food_size = fish_size / 10; for _ in 0..10 { let food_id = generate_unique_id(); let food = Food::new(food_id, fish.owner, food_size, food_value, ctx); Pond::add_food(food); } // remove fish Pond::remove_fish(fish.id); } ``` ### 6.6 Reward distribution ```move fun distribute_rewards(fish: &Fish::Fish, total_reward: u64) { let dev_reward = total_reward / 100; coin::mint<RGAS>(DEVELOPER_ADDRESS, dev_reward); let owner_reward = total_reward * 79/100; coin::mint<RGAS>(fish.owner, owner_reward); let feeder_reward = total_reward * 20 / 100; let total_food_value = 0u64; let mut contributions = Table::new<address, u64>(); for (feeder, value) in &fish.food_sources { total_food_value += *value; let entry = Table::get_mut_with_default(&mut contributions, *feeder, 0); *entry += *value; } if total_food_value > 0 { for (feeder, value) in Table::iter(&contributions) { let share = (*value * feeder_reward) / total_food_value; Player::add_reward(*feeder, share); } }
}
```

## 7. Test plan

### 7.1 Test cases

#### Use case 1: Stamina consumption and recovery

- **Steps**:
1. Create a fish, the initial stamina should be 10.
2. Move the fish once, the stamina should be reduced by 1.
3. Wait for 1 second, the stamina should be increased by 1.

- **Expected result**: Stamina is consumed and recovered correctly.

#### Use case 2: Fish movement and collision

- **Steps**:
1. Control the fish to move to the location with food.
2. Check the size and stamina of the fish.

- **Expected result**: The size of the fish increases, and the stamina is restored to 10.

#### Use case 3: Fish destruction and reward

- **Steps**:
1. Control the fish to move to the exit location.
2. Destroy the fish, check the player's balance and reward record.

- **Expected result**: Players get the correct rewards, and rewards for feeders are correctly distributed.

#### Use case 4: Fish death processing

- **Steps**:
1. Make the fish reach the maximum size limit.
2. Check whether the fish is overfed and whether food is generated.

- **Expected result**: The fish is overfed, food is generated, and rewards are correctly distributed.

### 7.2 Testing tools and environment

- Use the test framework of the Move language to write unit tests.
- Simulate multiple scenarios to ensure that the logic is correct.

## 8. Security considerations

- **Permission control**: Ensure that only the owner of the fish can control and destroy the fish.

- **Anti-cheating**: Prevent malicious players from tampering with key attributes such as physical strength and size.

- **Randomness security**: Use secure random number generation to prevent position prediction.

- **Prevent replay attacks**: Add transaction checks to prevent repeated operations.

## 9. Performance optimization

- **Data structure optimization**: Use efficient data structures (such as mappings and vectors) to manage fish and food.
- **Reduce storage usage**: Simplify data structure and reduce on-chain storage costs.
- **Batch operation**: Support batch feeding to reduce the number of transactions.

## 10. Future expansion

- **Multiple fish pond support**: Introduce fish ponds with different rules and rewards.
- **Special abilities**: Add special skills or attributes to fish.
- **Social functions**: Add player interaction, team formation and other functions.
- **Events and competitions**: Regularly hold in-game events to increase activity.

## 11. Conclusion

This technical solution describes the contract design and implementation method of the RoochFish game in detail, focusing on optimizing the food source tracking and reward distribution mechanism. By simplifying the data structure and logic, it not only ensures the fun and fairness of the game, but also reduces the complexity of implementation and maintenance. It is hoped that this solution can effectively guide development and create a popular blockchain game.