# FocEliza

A Move language implementation of Fully on-chain Eliza

## Project Overview

FocEliza demonstrates how to create and manage AI characters on blockchain, featuring:

* On-chain character creation and storage
* Dynamic character state updates
* On-chain memory system
* Character personality customization and evolution

## Features

✅ Implemented:
* Character creation and storage
* Character attribute updates
* Basic memory system

🚧 In Development:
* On-chain action system
* On-chain evolution mechanism
* On-chain decision-making system
* The character loader plugin for eliza

## Quick Start

### Prerequisites

- Install [Rooch CLI](https://rooch.network/learn/getting-started/installation)
- Running local Rooch node

### Build and Test

```sh
# Build the project
rooch move build -d

# Run tests
rooch move test
```

### Usage

1. Publish the module
```sh
rooch move publish --named-addresses foc_eliza=default
```
2. Create an on-chain character

```sh
rooch move run --function default::character::create_character_from_json --args file:../../../eliza/characters/dobby.character.json
```
> Note: Change the file path to your character file path.

```text
Execution info:
    status: Executed
    gas used: 1775707
    tx hash: 0x413f2aa79867ae7b8038265f17b837433917f53da3fed120e9b8e8166cf5bc38
    state root: 0x5c859b6ec2902a33da5b10ce47fb951281616cb97fef952a3099ef9a11e3cc9b
    event root: 0x9664ae38517e0827890ec55d20256679ff612288551bedffb13dd2a7aa738a6c

New objects:
    objectId: 0x8604bfaa406b4eae756e9eaf710c074573e18feae6f8c974df1fc9b0259e7e62
    type    : 0x285529d7fd13ffcda9d89cd250b4025ba9226c0e2e57f5ca3d739cb236dc259d::agent_cap::AgentCap

    objectId: 0xd858ebbc8e0e5c2128800b9a715e3bd8ceae2fb8a75df5cc40b58b86f1dc77ee
    type    : 0x285529d7fd13ffcda9d89cd250b4025ba9226c0e2e57f5ca3d739cb236dc259d::character::Character
```

3. Query the character information

```sh
rooch object -i 0xd858ebbc8e0e5c2128800b9a715e3bd8ceae2fb8a75df5cc40b58b86f1dc77ee
```
> Note: Replace the `-i` argument with your Character objectId.

```json
{
  "data": [
    {
      "id": "0xd858ebbc8e0e5c2128800b9a715e3bd8ceae2fb8a75df5cc40b58b86f1dc77ee",
      "owner": "rooch19p2jn4laz0lum2wcnnf9pdqztw5jymqw9etltj3awwwtydkuykwsas4mgf",
      "owner_bitcoin_address": "bcrt1p56tdhxkcpc5xvdurfnufn9lkkywsh0gxttv5ktkvlezj0t23nasqawwrla",
      "flag": 0,
      "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
      "size": "0",
      "created_at": "1736767738531",
      "updated_at": "1736767738531",
      "object_type": "0x285529d7fd13ffcda9d89cd250b4025ba9226c0e2e57f5ca3d739cb236dc259d::character::Character",
      "value": "0x0005446f62627900000009616e7468726f70696300000000044c446f6262792069732061206672656520617373697374616e742077686f2063686f6f73657320746f2068656c702062656361757365206f662068697320656e6f726d6f75732068656172742e4045787472656d656c79206465766f74656420616e642077696c6c20676f20746f20616e79206c656e67746820746f2068656c702068697320667269656e64732e4d537065616b7320696e20746869726420706572736f6e20616e6420686173206120756e697175652c20656e64656172696e6720776179206f662065787072657373696e672068696d73656c662e5b4b6e6f776e20666f72206869732063726561746976652070726f626c656d2d736f6c76696e672c206576656e2069662068697320736f6c7574696f6e732061726520736f6d6574696d657320756e636f6e76656e74696f6e616c2e04514f6e6365206120686f7573652d656c662c206e6f77206120667265652068656c7065722077686f2063686f6f73657320746f207365727665206f7574206f66206c6f766520616e64206c6f79616c74792e4246616d6f757320666f72206869732064656469636174696f6e20746f2068656c70696e6720486172727920506f7474657220616e642068697320667269656e64732e454b6e6f776e20666f72206869732063726561746976652c20696620736f6d6574696d6573206472616d617469632c20736f6c7574696f6e7320746f2070726f626c656d732e3856616c7565732066726565646f6d206275742063686f6f73657320746f2068656c702074686f73652068652063617265732061626f75742e0254446f6262792072656d696e647320667269656e64732074686174206576656e2074686520736d616c6c6573742068656c7065722063616e206d616b6520746865206269676765737420646966666572656e63652170446f62627920736179733a20275768656e20696e20646f7562742c207472792074686520756e636f6e76656e74696f6e616c20736f6c7574696f6e2127202842757420446f626279206164766973657320746f206265206361726566756c207769746820666c79696e672063617273290100050c456e74687573696173746963054c6f79616c1354686972642d706572736f6e207370656563680843726561746976650a50726f746563746976650405456167657209456e64656172696e67074465766f74656411536c696768746c79206472616d61746963050c54686972642d706572736f6e0c456e746875736961737469630748656c7066756c0b456e636f75726167696e6706517569726b7907054c6f79616c0c456e74687573696173746963084372656174697665074465766f7465640d467265652d73706972697465640a50726f746563746976650e556e636f6e76656e74696f6e616c05174d616769632028686f7573652d656c66207374796c65291843726561746976652070726f626c656d2d736f6c76696e671350726f74656374697665207365727669636573104c6f79616c20617373697374616e636518556e636f6e76656e74696f6e616c20736f6c7574696f6e7300",
      "decoded_value": {
        "abilities": 8,
        "type": "0x285529d7fd13ffcda9d89cd250b4025ba9226c0e2e57f5ca3d739cb236dc259d::character::Character",
        "value": {
          "adjectives": {
            "abilities": 7,
            "type": "0x1::string::String",
            "field": [
              "bytes"
            ],
            "value": [
              [
                "0x0000000000000000000000000000000000000000000000000000004c6f79616c"
              ],
              [
                "0x0000000000000000000000000000000000000000456e74687573696173746963"
              ],
              [
                "0x0000000000000000000000000000000000000000000000004372656174697665"
              ],
              [
                "0x000000000000000000000000000000000000000000000000004465766f746564"
              ],
              [
                "0x00000000000000000000000000000000000000467265652d7370697269746564"
              ],
              [
                "0x0000000000000000000000000000000000000000000050726f74656374697665"
              ],
              [
                "0x000000000000000000000000000000000000556e636f6e76656e74696f6e616c"
              ]
            ]
          },
          "bio": {
            "abilities": 7,
            "type": "0x1::string::String",
            "field": [
              "bytes"
            ],
            "value": [
              [
                "0x446f6262792069732061206672656520617373697374616e742077686f2063686f6f73657320746f2068656c702062656361757365206f662068697320656e6f726d6f75732068656172742e"
              ],
              [
                "0x45787472656d656c79206465766f74656420616e642077696c6c20676f20746f20616e79206c656e67746820746f2068656c702068697320667269656e64732e"
              ],
              [
                "0x537065616b7320696e20746869726420706572736f6e20616e6420686173206120756e697175652c20656e64656172696e6720776179206f662065787072657373696e672068696d73656c662e"
              ],
              [
                "0x4b6e6f776e20666f72206869732063726561746976652070726f626c656d2d736f6c76696e672c206576656e2069662068697320736f6c7574696f6e732061726520736f6d6574696d657320756e636f6e76656e74696f6e616c2e"
              ]
            ]
          },
          "clients": [],
          "id": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": []
            }
          },
          "imageModelProvider": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": []
            }
          },
          "imageVisionModelProvider": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": []
            }
          },
          "knowledge": {
            "abilities": 7,
            "type": "0x1::string::String",
            "field": [
              "bytes"
            ],
            "value": [
              [
                "0x0000000000000000004d616769632028686f7573652d656c66207374796c6529"
              ],
              [
                "0x000000000000000043726561746976652070726f626c656d2d736f6c76696e67"
              ],
              [
                "0x0000000000000000000000000050726f74656374697665207365727669636573"
              ],
              [
                "0x000000000000000000000000000000004c6f79616c20617373697374616e6365"
              ],
              [
                "0x0000000000000000556e636f6e76656e74696f6e616c20736f6c7574696f6e73"
              ]
            ]
          },
          "lore": {
            "abilities": 7,
            "type": "0x1::string::String",
            "field": [
              "bytes"
            ],
            "value": [
              [
                "0x4f6e6365206120686f7573652d656c662c206e6f77206120667265652068656c7065722077686f2063686f6f73657320746f207365727665206f7574206f66206c6f766520616e64206c6f79616c74792e"
              ],
              [
                "0x46616d6f757320666f72206869732064656469636174696f6e20746f2068656c70696e6720486172727920506f7474657220616e642068697320667269656e64732e"
              ],
              [
                "0x4b6e6f776e20666f72206869732063726561746976652c20696620736f6d6574696d6573206472616d617469632c20736f6c7574696f6e7320746f2070726f626c656d732e"
              ],
              [
                "0x56616c7565732066726565646f6d206275742063686f6f73657320746f2068656c702074686f73652068652063617265732061626f75742e"
              ]
            ]
          },
          "modelEndpointOverride": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": []
            }
          },
          "modelProvider": "anthropic",
          "name": "Dobby",
          "plugins": [],
          "postExamples": {
            "abilities": 7,
            "type": "0x1::string::String",
            "field": [
              "bytes"
            ],
            "value": [
              [
                "0x446f6262792072656d696e647320667269656e64732074686174206576656e2074686520736d616c6c6573742068656c7065722063616e206d616b6520746865206269676765737420646966666572656e636521"
              ],
              [
                "0x446f62627920736179733a20275768656e20696e20646f7562742c207472792074686520756e636f6e76656e74696f6e616c20736f6c7574696f6e2127202842757420446f626279206164766973657320746f206265206361726566756c207769746820666c79696e67206361727329"
              ]
            ]
          },
          "style": {
            "abilities": 7,
            "type": "0x285529d7fd13ffcda9d89cd250b4025ba9226c0e2e57f5ca3d739cb236dc259d::types::Style",
            "value": {
              "all": {
                "abilities": 7,
                "type": "0x1::string::String",
                "field": [
                  "bytes"
                ],
                "value": [
                  [
                    "0x0000000000000000000000000000000000000000456e74687573696173746963"
                  ],
                  [
                    "0x0000000000000000000000000000000000000000000000000000004c6f79616c"
                  ],
                  [
                    "0x0000000000000000000000000054686972642d706572736f6e20737065656368"
                  ],
                  [
                    "0x0000000000000000000000000000000000000000000000004372656174697665"
                  ],
                  [
                    "0x0000000000000000000000000000000000000000000050726f74656374697665"
                  ]
                ]
              },
              "chat": {
                "abilities": 7,
                "type": "0x1::string::String",
                "field": [
                  "bytes"
                ],
                "value": [
                  [
                    "0x0000000000000000000000000000000000000000000000000000004561676572"
                  ],
                  [
                    "0x0000000000000000000000000000000000000000000000456e64656172696e67"
                  ],
                  [
                    "0x000000000000000000000000000000000000000000000000004465766f746564"
                  ],
                  [
                    "0x000000000000000000000000000000536c696768746c79206472616d61746963"
                  ]
                ]
              },
              "post": {
                "abilities": 7,
                "type": "0x1::string::String",
                "field": [
                  "bytes"
                ],
                "value": [
                  [
                    "0x000000000000000000000000000000000000000054686972642d706572736f6e"
                  ],
                  [
                    "0x0000000000000000000000000000000000000000456e74687573696173746963"
                  ],
                  [
                    "0x0000000000000000000000000000000000000000000000000048656c7066756c"
                  ],
                  [
                    "0x000000000000000000000000000000000000000000456e636f75726167696e67"
                  ],
                  [
                    "0x0000000000000000000000000000000000000000000000000000517569726b79"
                  ]
                ]
              }
            }
          },
          "system": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": []
            }
          },
          "topics": {
            "abilities": 7,
            "type": "0x1::string::String",
            "field": [
              "bytes"
            ],
            "value": [
              [
                "0x0000000000000000000000000000000000000000000000000000000000000000"
              ]
            ]
          },
          "twitterProfile": {
            "abilities": 7,
            "type": "0x1::option::Option<0x285529d7fd13ffcda9d89cd250b4025ba9226c0e2e57f5ca3d739cb236dc259d::types::TwitterProfile>",
            "value": {
              "vec": []
            }
          },
          "username": {
            "abilities": 7,
            "type": "0x1::option::Option<0x1::string::String>",
            "value": {
              "vec": []
            }
          }
        }
      },
      "tx_order": "0",
      "state_index": "0",
      "display_fields": null
    }
  ],
  "next_cursor": {
    "tx_order": "0",
    "state_index": "0"
  },
  "has_next_page": false
}
```

4. Update the character via add a bio

```bash
rooch move run --function default::character::add_bio_entry --args object:<your-AgentCap-object-id> --args string:"Bobby is a programmer"
```
```text
Execution info:
    status: Executed
    gas used: 280496
    tx hash: 0x848a79967410c0e74887d1e47f45f3e013e41d0ea37435251fed4df982dc035f
    state root: 0xb9ba7a958eb1e63683daadb7cfe9f22fa731019b8fff7b6e76740e0b70d97174
    event root: 0x6b25ec6fd717296603fb0818f24682d88bfdf9c55d995091865c582eed18dc71
```

### Why On-Chain?

Storing AI characters and their memory on-chain provides three key advantages:

1. **Dynamic Evolution**
- Characters evolve through conversations
- Bio, interests, and templates update dynamically
- Transparent growth process with community oversight

2. **Governance & Auditing**
- Community-managed memory system
- Prevents memory contamination
- Public behavior auditing
- Alignment with community standards

3. **Decentralized Trust**
- Enhanced trustworthiness
- Fair and open AI ecosystem
- Transparent character development

### Roadmap

1. **Character System**
- Dynamic character loading
- Real-time definition updates

2. **Memory System**
- On-chain memory sync
- Real-time state updates
- Interaction history

3. **AI Integration**
- On-chain AI Oracle
- Decision-making capabilities
- Smart contract inference

4. **Framework Development**
- Standardized components
- Move framework integration
- Developer tools

### Get Involved

1. **Core Development**
- On-chain vector operations
- AI inference mechanisms
- Smart contract innovations

2. **Service Building**
- AI Agent social services
- Multi-user chatrooms
- Character interaction systems

3. **Integration**
- AI + DeFi applications
- BTCFi integration
- Cross-chain capabilities
