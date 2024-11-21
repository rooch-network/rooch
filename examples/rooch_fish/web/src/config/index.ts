export const config = {
  debug: false,
  network: "testnet", // localnet, testnet
  roochFishAddress: "0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8",
  gameStateObjectID: "0x3bbe1056fcfdfee92eae4d7045d9a231a3d5a4ca7ddd77e43d730bc80d8bc4d8",
  ponds: {
    0: "0x4afb602f8bbbb516ef9c8dbcecb8d91bfb5ddb618a7505da0abb1904ebc1784b",
    1: "0x1fd948251d394db82f9d4a1e82118cf232c0f4f9c2c7ffa45ea9dffad28e9412",
    2: "0x676cb4ef030fd3cfc4a151a348e6c9bc987f97a11586dcee5660cd6bb1a2047c",
    3: "0x14b8edb38062592b47b6898e83c7b0df8f1d4439c0fb9d6728435ef870516b15",
    4: "0x2b3d3dc14e62060099e49a8c522b5698219795bf6a87d42624cc69eaf6263627",
    5: "0x7fe629442f293a0307c7cc6ae12edc0c60c4a7708c75a83b5f671ff03544e781",
    6: "0x855d118b9da7ac1f32922cf568f51d9df1e28cd0205d9f241ba0007c22fea5bb",
    7: "0xac8652636629f6bfe5ac6e102308bc032b01101243e50a3604c3d12955abf179"
  }
}

export type PondID = keyof typeof config.ponds;

/*
testnet

New modules:
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::simple_rng
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::fish
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::player
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::quad_tree
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::utils
    0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::food

gameworld:
[
  {
    "id": "0x3bbe1056fcfdfee92eae4d7045d9a231a3d5a4ca7ddd77e43d730bc80d8bc4d8",
    "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
    "owner_bitcoin_address": null,
    "flag": 1,
    "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
    "size": "0",
    "created_at": "1732199736531",
    "updated_at": "1732199736531",
    "object_type": "0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish::GameState",
    "value": "0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e801ae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b601ed4d53e3041bc78d727d34f30ccdaaa4227cadb87cd6ca6f6182673ba94b509c00000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "decoded_value": {
      "abilities": 8,
      "type": "0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::rooch_fish::GameState",
      "value": {
        "admin": "0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8",
        "player_list": {
          "abilities": 12,
          "type": "0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::player::PlayerList",
          "value": {
            "player_count": "0",
            "players": {
              "abilities": 4,
              "type": "0x2::table::Table<address, 0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::player::PlayerState>",
              "value": {
                "handle": {
                  "abilities": 12,
                  "type": "0x2::object::Object<0x2::table::TablePlaceholder>",
                  "value": {
                    "id": "0xed4d53e3041bc78d727d34f30ccdaaa4227cadb87cd6ca6f6182673ba94b509c"
                  }
                }
              }
            },
            "total_feed": "0"
          }
        },
        "ponds": {
          "abilities": 4,
          "type": "0x2::table::Table<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "handle": {
              "abilities": 12,
              "type": "0x2::object::Object<0x2::table::TablePlaceholder>",
              "value": {
                "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b6"
              }
            }
          }
        }
      }
    },
    "display_fields": null
  }
]

ponds:
{
  "data": [
    {
      "field_key": "0x11228d102ec1ccd0e71d31a6115c33b886a4cd5cb48113db5c851c6d96e82b7f",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b611228d102ec1ccd0e71d31a6115c33b886a4cd5cb48113db5c851c6d96e82b7f",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x070000000000000001ac8652636629f6bfe5ac6e102308bc032b01101243e50a3604c3d12955abf179",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "7",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0xac8652636629f6bfe5ac6e102308bc032b01101243e50a3604c3d12955abf179"
              }
            }
          }
        },
        "display_fields": null
      }
    },
    {
      "field_key": "0x14e41f37bef443dbf4163625571e6d035763b553cad649387b41be8f7c4c7569",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b614e41f37bef443dbf4163625571e6d035763b553cad649387b41be8f7c4c7569",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x020000000000000001676cb4ef030fd3cfc4a151a348e6c9bc987f97a11586dcee5660cd6bb1a2047c",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "2",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0x676cb4ef030fd3cfc4a151a348e6c9bc987f97a11586dcee5660cd6bb1a2047c"
              }
            }
          }
        },
        "display_fields": null
      }
    },
    {
      "field_key": "0x28b56697e9fd6fbf8d6513b44cbeb793537ef7bd71dbe03c7a8d5cfc992ff407",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b628b56697e9fd6fbf8d6513b44cbeb793537ef7bd71dbe03c7a8d5cfc992ff407",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x0000000000000000014afb602f8bbbb516ef9c8dbcecb8d91bfb5ddb618a7505da0abb1904ebc1784b",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "0",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0x4afb602f8bbbb516ef9c8dbcecb8d91bfb5ddb618a7505da0abb1904ebc1784b"
              }
            }
          }
        },
        "display_fields": null
      }
    },
    {
      "field_key": "0x2ce2e7bbb14b9b8d582aaf87c705f424e63373a1bb602deed89e14c95615e4b6",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b62ce2e7bbb14b9b8d582aaf87c705f424e63373a1bb602deed89e14c95615e4b6",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x0400000000000000012b3d3dc14e62060099e49a8c522b5698219795bf6a87d42624cc69eaf6263627",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "4",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0x2b3d3dc14e62060099e49a8c522b5698219795bf6a87d42624cc69eaf6263627"
              }
            }
          }
        },
        "display_fields": null
      }
    },
    {
      "field_key": "0x3bc998c5c75500958ca88ca47f5b654ff2fa97da0c1539b6b4c4c8722975a80f",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b63bc998c5c75500958ca88ca47f5b654ff2fa97da0c1539b6b4c4c8722975a80f",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x060000000000000001855d118b9da7ac1f32922cf568f51d9df1e28cd0205d9f241ba0007c22fea5bb",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "6",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0x855d118b9da7ac1f32922cf568f51d9df1e28cd0205d9f241ba0007c22fea5bb"
              }
            }
          }
        },
        "display_fields": null
      }
    },
    {
      "field_key": "0x5a473312d437bbf8c2690476f7f1df5040ff1f2398d19d7c039858dd3423bbbc",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b65a473312d437bbf8c2690476f7f1df5040ff1f2398d19d7c039858dd3423bbbc",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x0500000000000000017fe629442f293a0307c7cc6ae12edc0c60c4a7708c75a83b5f671ff03544e781",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "5",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0x7fe629442f293a0307c7cc6ae12edc0c60c4a7708c75a83b5f671ff03544e781"
              }
            }
          }
        },
        "display_fields": null
      }
    },
    {
      "field_key": "0x7eb4036673c8611e43c3eff1202446612f22a4b3bac92b7e14c0562ade5f1a3f",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b67eb4036673c8611e43c3eff1202446612f22a4b3bac92b7e14c0562ade5f1a3f",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x0100000000000000011fd948251d394db82f9d4a1e82118cf232c0f4f9c2c7ffa45ea9dffad28e9412",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "1",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0x1fd948251d394db82f9d4a1e82118cf232c0f4f9c2c7ffa45ea9dffad28e9412"
              }
            }
          }
        },
        "display_fields": null
      }
    },
    {
      "field_key": "0xf30b43fa7dbdf31380c5b3efb2e960569b6caa30e6ee302358fbd43a605a5dbc",
      "state": {
        "id": "0xae308aa50bded0b341120431a9049ad1f151b345115a17bcefcad943505fa3b6f30b43fa7dbdf31380c5b3efb2e960569b6caa30e6ee302358fbd43a605a5dbc",
        "owner": "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen",
        "owner_bitcoin_address": null,
        "flag": 0,
        "state_root": "0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000",
        "size": "0",
        "created_at": "1732199736531",
        "updated_at": "1732199736531",
        "object_type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
        "value": "0x03000000000000000114b8edb38062592b47b6898e83c7b0df8f1d4439c0fb9d6728435ef870516b15",
        "decoded_value": {
          "abilities": 12,
          "type": "0x2::object::DynamicField<u64, 0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>>",
          "value": {
            "name": "3",
            "value": {
              "abilities": 12,
              "type": "0x2::object::Object<0xb38a327121ab8e9091a04377ec1e9af9ab4b801dbfb368f20fb0c080c763f7e8::pond::PondState>",
              "value": {
                "id": "0x14b8edb38062592b47b6898e83c7b0df8f1d4439c0fb9d6728435ef870516b15"
              }
            }
          }
        },
        "display_fields": null
      }
    }
  ],
  "next_cursor": "0xf30b43fa7dbdf31380c5b3efb2e960569b6caa30e6ee302358fbd43a605a5dbc",
  "has_next_page": false
}
*/