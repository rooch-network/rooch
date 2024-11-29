## Game Design: Have You Caught a Fish (Optimized Version)

### 1. Core Concept

"Have You Caught a Fish" is a multiplayer online game based on the Rooch blockchain, where players compete by buying and controlling virtual fish in multiple dynamic fish ponds. The game combines elements of growth, strategy, and economy. Players can grow by eating other fish or collecting food, and earn token rewards under specific conditions. Players can also feed fish ponds with food, influencing the game economy and potentially gaining profits. The game introduces a stamina system, adding depth and balance to the strategy.

### 2. Game Mechanics and Rules

#### 2.1 Fish Pond System

- **Multi-Pond Design**: The game includes multiple fish ponds, each with different sizes, capacities, and purchase prices.
- **Pond Parameters**: Each pond has fixed width, height, maximum number of fish, and maximum amount of food.
- **Maximum Fish Size Limit**: Each pond sets a maximum fish size limit. When a fish reaches or exceeds this limit, it will "burst" and disappear, generating a certain amount of food (e.g., 10 pieces), which will be randomly distributed in the pond.
- **Exit Area**: Each pond has a specific exit area where fish can be destroyed for rewards.

#### 2.2 Purchase and Generation

- **Buying Fish**: Players use RGAS tokens to buy fish in specific ponds, with different purchase prices for each pond.
- **Random Generation**: New fish are randomly generated in the pond with a fixed initial size.
- **New Fish Protection Mechanism**: Newly born fish are protected for 1 minute and cannot be eaten by other fish.

#### 2.3 Movement and Growth

- **Movement Control**:
  - Players can control the fish to move up, down, left, or right in the pond.
  - Each movement consumes 1 stamina point.
  - When stamina is 0, the fish cannot move.
- **Stamina System**:
  - Each fish has a stamina attribute with a maximum value of 10 points.
  - Stamina recovers 1 point per second.
  - Stamina cannot exceed the maximum value of 10 points.
  - Eating food or smaller fish immediately restores 10 stamina points.
- **Growth Mechanism**:
  - Eating smaller fish increases size and immediately restores 10 stamina points.
  - Eating food in the pond increases size and immediately restores 10 stamina points.
  - When a fish reaches the maximum size limit, it will "burst" and turn into food.
- **Boundary Limits**: Fish movement is restricted by pond boundaries.

#### 2.4 Feeding and Food System

- **Feeding Food**:
  - Players can use RGAS tokens to feed food into the pond.
  - The amount of food fed affects the total amount of food in the pond.
- **Food Generation**: Food is generated at random positions in the pond, with a maximum quantity.

#### 2.5 Interaction and Competition

- **Eating Rules**: Fish can only eat smaller fish, which immediately restores 10 stamina points.
- **Food Consumption**: Eating food increases size and immediately restores 10 stamina points.
- **New Fish Protection**: New fish are protected for 1 minute after birth and cannot be eaten by other fish.
- **Collision Detection**: The system automatically detects collisions between fish and fish, fish and food, and processes the results.

#### 2.6 Exit and Rewards

- **Exit Mechanism**: Fish can move to a specific exit area in the pond to be destroyed.
- **Token Rewards**:
  - When a fish is destroyed at the exit, the player receives RGAS token rewards based on the fish's size.
  - Reward Distribution:
    - **1%** of the reward is allocated to the developers for supporting ongoing development and maintenance.
    - **20%** of the reward is allocated to players who fed food, based on the proportion of food fed by each player relative to the total food eaten by the fish.
    - The remaining reward goes to the owner of the fish.

#### 2.7 Economic System

- **Global Economy**: The game maintains a global player list, recording total feeding amounts and player numbers.
- **Pond Economy**: Each pond has an independent economic system, including player lists and total feeding amounts.
- **Stamina Management**: Players need to manage their fish's stamina effectively, balancing movement and rest.

### 3. Player Objectives and Strategies

#### 3.1 Main Objectives

- **Maximize Profits**: Grow large fish and destroy them at the right time to maximize RGAS token rewards.
- **Survival and Growth**: Survive and become the largest fish in a competitive environment.

#### 3.2 Strategic Considerations

- **Pond Selection**: Choose the appropriate pond based on your strategy and funds.
- **Investment Balance**: Balance buying fish, feeding food, and obtaining rewards.
- **Risk Management**: Avoid being eaten by larger fish during growth.
- **Timing**: Choose the right time to move fish to the exit area for rewards.
- **Maximum Size Management**: Decide whether to move fish to the exit area for tokens when approaching the maximum size, or risk further growth for more food.
- **Stamina Management**: Plan movement paths to avoid running out of stamina and being unable to evade danger or hunt.
- **Rest Strategy**: Rest fish in safe areas to recover stamina for future rapid movement.
- **Stamina Recovery Strategy**: Weigh the risk of eating food or smaller fish to quickly recover stamina.
- **Offense-Defense Balance**: Choose between chasing prey and conserving stamina, as successful hunting immediately restores stamina.
- **Risk Assessment**: Assess whether it's worth expending stamina to chase food or smaller fish, considering the immediate stamina recovery upon success.

### 4. Technical Implementation

#### 4.1 Smart Contracts

- Use Move language to write smart contracts, implementing core logic for ponds, fish, and food.
- Implement key functions such as purchase, movement, growth, destruction, and feeding.
- Add stamina value to fish attributes and implement stamina consumption and recovery logic.
- Include stamina checks in the movement function to ensure movement only when there is enough stamina.
- Add immediate stamina recovery in the logic for eating food or smaller fish.
- Ensure stamina recovery does not exceed the maximum value of 10 points.
- Ensure contract security and efficiency.

#### 4.2 Frontend Development

- Develop an intuitive user interface to display multiple ponds and game status.
- Implement real-time updates and interaction functions.
- Display the current stamina value of each fish in the user interface.
- Implement visual feedback for stamina recovery, allowing players to clearly know when they can move again.
- Implement visual effects for rapid stamina recovery when eating food or smaller fish, allowing players to clearly perceive stamina changes.
- Consider adding a special animation or sound effect to emphasize rapid stamina recovery.
- Integrate blockchain wallets to simplify token operations.

#### 4.3 Backend Services

- Develop backend services to handle high-frequency updates and complex calculations.
- Implement game state synchronization and persistence.
- Implement a timer function for automatic stamina recovery.
- Ensure real-time synchronization and updating of stamina values.
- Optimize the stamina recovery calculation logic to balance normal and rapid recovery.
- Implement relevant data statistics, such as the number of times players recover stamina by eating food/smaller fish, for subsequent game balance adjustments.

### 5. Future Extensions

- Introduce new pond types to increase game diversity.
- Implement special abilities or attribute systems for fish.
- Add social features such as teaming and guilds.
- Host regular events or competitions to increase game fun.
- Adjust stamina system parameters based on player feedback and data analysis to optimize game balance.
- Consider introducing more stamina-related special items or skills to increase strategic depth.
