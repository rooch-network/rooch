# RoochFish

"Have You Caught Fish" is an open-source full-chain game based on the Rooch blockchain, where players can buy and control virtual fish in multiple dynamic fish ponds. The fish grow by eating other fish and food, and players can earn token rewards under specific conditions. The game introduces a stamina system, adding strategic depth and balance. The smart contract part of the project is open-source, aiming to provide developers and players with a transparent and scalable gaming environment.

## Detailed Documentation

- [Game Design Document](docs/game_design.md) - Detailed game mechanics, rules, and gameplay instructions
- [Technical Design Document](docs/tech_design.md) - Technical implementation and architecture design

## Core Features

### Multi-Pond System
- Multiple ponds with different sizes, capacities, and prices
- Each pond has a maximum fish size limit
- Specific exit areas for fish destruction and reward acquisition

### Stamina System
- Each fish has a maximum stamina of 10 points
- Each move consumes 1 stamina point
- Stamina recovers 1 point per second
- Eating food or smaller fish immediately restores 10 stamina points

### Economic System
- Use RGAS tokens to buy fish and feed food
- Rewards distribution upon fish destruction:
  - 1% goes to the developer
  - 20% goes to the player who fed the food
  - The remaining reward goes to the fish owner

### Game Mechanics
- New fish have a 1-minute protection period
- Fish can only eat smaller fish
- Fish "burst" and turn into food when they reach the maximum size limit
- Comprehensive collision detection system

## Quick Start

### Environment Setup

1. **Install Rooch CLI**:
   ```bash
   # Refer to the Rooch official documentation to install the CLI
   ```

### Using Makefile

The project provides the following Makefile commands:

```bash
make build    # Build the contract
make publish  # Publish the contract
make test     # Run tests
make debug    # Run a specific test
make clean    # Clean build files
```

## Project Structure

```
rooch-fish/
├── docs/           # Project documentation
│   ├── game_design.md    # Game design document
│   └── tech_design.md    # Technical design document
├── sources/        # Move contract source code
├── tests/          # Test files
├── build/          # Build output
├── Move.toml       # Project configuration file
└── Makefile        # Build script
```

## Strategic Tips

- **Pond Selection**: Choose the appropriate pond based on strategy and funds
- **Stamina Management**: Plan movement paths wisely, rest in safe areas to recover stamina
- **Risk Assessment**: Balance the pursuit of food/smaller fish with stamina preservation
- **Optimal Timing**: Move fish to exit areas at the right time to claim rewards
- **Investment Balance**: Balance buying fish, feeding food, and claiming rewards

## Future Plans

- Introduce new pond types
- Implement special abilities for fish
- Add social features (team, guild)
- Host regular events and competitions
- Continuously optimize the stamina system balance
