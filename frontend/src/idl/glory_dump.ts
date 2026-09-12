/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/glory_dump.json`.
 */
export type GloryDump = {
  "address": "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS",
  "metadata": {
    "name": "gloryDump",
    "version": "3.0.0",
    "spec": "0.1.0",
    "description": "Solana program for the GLORY/DUMP reverse-wealth strategy game",
    "repository": "https://github.com/Jabari-Byrd/Glory-Dump"
  },
  "instructions": [
    {
      "name": "absorb",
      "discriminator": [
        121,
        23,
        131,
        56,
        87,
        131,
        143,
        148
      ],
      "accounts": [
        {
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "actorPlayer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "actorPlayer.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "targetPlayer",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "targetPlayer.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "actorLane",
          "writable": true
        },
        {
          "name": "targetLane",
          "writable": true
        },
        {
          "name": "rivalry",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  105,
                  118,
                  97,
                  108,
                  114,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "actorPlayer.owner",
                "account": "playerEpoch"
              },
              {
                "kind": "account",
                "path": "targetPlayer.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "destinationIndex",
          "type": "u8"
        },
        {
          "name": "sourceIndex",
          "type": "u8"
        }
      ]
    },
    {
      "name": "armRedirect",
      "discriminator": [
        167,
        37,
        252,
        180,
        42,
        61,
        67,
        243
      ],
      "accounts": [
        {
          "name": "signer",
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "player.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "lane",
          "writable": true
        }
      ],
      "args": [
        {
          "name": "laneIndex",
          "type": "u8"
        }
      ]
    },
    {
      "name": "authorizeSession",
      "discriminator": [
        187,
        218,
        251,
        161,
        99,
        40,
        34,
        34
      ],
      "accounts": [
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "delegate",
          "type": "pubkey"
        },
        {
          "name": "durationSeconds",
          "type": "i64"
        },
        {
          "name": "maxActions",
          "type": "u16"
        }
      ]
    },
    {
      "name": "beginActive",
      "discriminator": [
        40,
        215,
        228,
        153,
        64,
        13,
        32,
        10
      ],
      "accounts": [
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "beginReveal",
      "discriminator": [
        161,
        44,
        44,
        166,
        5,
        11,
        216,
        150
      ],
      "accounts": [
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "beginSettlement",
      "discriminator": [
        124,
        223,
        80,
        249,
        19,
        59,
        26,
        92
      ],
      "accounts": [
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "cancelEpoch",
      "discriminator": [
        120,
        226,
        234,
        25,
        215,
        85,
        0,
        155
      ],
      "accounts": [
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "claimAllocation",
      "discriminator": [
        19,
        148,
        128,
        46,
        220,
        171,
        177,
        43
      ],
      "accounts": [
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "laneZero",
          "writable": true
        },
        {
          "name": "laneOne",
          "writable": true
        },
        {
          "name": "laneTwo",
          "writable": true
        },
        {
          "name": "laneThree",
          "writable": true
        }
      ],
      "args": []
    },
    {
      "name": "claimBond",
      "discriminator": [
        173,
        34,
        157,
        61,
        45,
        120,
        246,
        11
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "claimKeeperReward",
      "discriminator": [
        84,
        112,
        54,
        166,
        64,
        240,
        183,
        119
      ],
      "accounts": [
        {
          "name": "keeper",
          "writable": true,
          "signer": true
        },
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "keeperCredit",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  101,
                  112,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "keeper"
              }
            ]
          }
        },
        {
          "name": "gloryMint",
          "writable": true
        },
        {
          "name": "destination",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "keeper"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "gloryMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "claimPlayerReward",
      "discriminator": [
        222,
        15,
        251,
        15,
        100,
        23,
        110,
        94
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "leaderboard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  101,
                  97,
                  100,
                  101,
                  114,
                  98,
                  111,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "gloryMint",
          "writable": true
        },
        {
          "name": "destination",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "gloryMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "closeKeeperCredit",
      "discriminator": [
        244,
        149,
        54,
        208,
        90,
        249,
        223,
        50
      ],
      "accounts": [
        {
          "name": "keeper",
          "writable": true,
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "keeperCredit",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  101,
                  112,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "keeper"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "closePlayerAccounts",
      "discriminator": [
        149,
        145,
        187,
        182,
        79,
        164,
        118,
        36
      ],
      "accounts": [
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "leaderboard",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  101,
                  97,
                  100,
                  101,
                  114,
                  98,
                  111,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "laneZero",
          "writable": true
        },
        {
          "name": "laneOne",
          "writable": true
        },
        {
          "name": "laneTwo",
          "writable": true
        },
        {
          "name": "laneThree",
          "writable": true
        }
      ],
      "args": []
    },
    {
      "name": "closeRivalry",
      "discriminator": [
        222,
        86,
        73,
        188,
        84,
        215,
        169,
        8
      ],
      "accounts": [
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "rentRecipient",
          "writable": true
        },
        {
          "name": "rivalry",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  105,
                  118,
                  97,
                  108,
                  114,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "rivalry.actor",
                "account": "rivalry"
              },
              {
                "kind": "account",
                "path": "rivalry.target",
                "account": "rivalry"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "completeEpoch",
      "discriminator": [
        75,
        152,
        53,
        199,
        144,
        192,
        221,
        207
      ],
      "accounts": [
        {
          "name": "finalizer",
          "signer": true
        },
        {
          "name": "protocol",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "leaderboard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  101,
                  97,
                  100,
                  101,
                  114,
                  98,
                  111,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "dump",
      "discriminator": [
        50,
        220,
        141,
        15,
        162,
        154,
        137,
        242
      ],
      "accounts": [
        {
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "actorPlayer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "actorPlayer.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "targetPlayer",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "targetPlayer.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "actorLane",
          "writable": true
        },
        {
          "name": "targetLane",
          "writable": true
        },
        {
          "name": "rivalry",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  105,
                  118,
                  97,
                  108,
                  114,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "actorPlayer.owner",
                "account": "playerEpoch"
              },
              {
                "kind": "account",
                "path": "targetPlayer.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "sourceIndex",
          "type": "u8"
        },
        {
          "name": "targetIndex",
          "type": "u8"
        }
      ]
    },
    {
      "name": "initializeProtocol",
      "discriminator": [
        188,
        233,
        252,
        106,
        134,
        146,
        202,
        91
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "protocol",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "gloryMint",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  108,
                  111,
                  114,
                  121,
                  95,
                  109,
                  105,
                  110,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "const",
                "value": [
                  1,
                  0,
                  0,
                  0,
                  0,
                  0,
                  0,
                  0
                ]
              }
            ]
          }
        },
        {
          "name": "leaderboard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  101,
                  97,
                  100,
                  101,
                  114,
                  98,
                  111,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "const",
                "value": [
                  1,
                  0,
                  0,
                  0,
                  0,
                  0,
                  0,
                  0
                ]
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "openNextEpoch",
      "discriminator": [
        141,
        217,
        180,
        18,
        234,
        118,
        50,
        92
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "protocol",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "previousEpoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "protocol.currentEpoch",
                "account": "protocol"
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "arg",
                "path": "nextEpoch"
              }
            ]
          }
        },
        {
          "name": "leaderboard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  101,
                  97,
                  100,
                  101,
                  114,
                  98,
                  111,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "nextEpoch"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "nextEpoch",
          "type": "u64"
        }
      ]
    },
    {
      "name": "refreshBadges",
      "discriminator": [
        55,
        232,
        61,
        73,
        253,
        99,
        155,
        36
      ],
      "accounts": [
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "register",
      "discriminator": [
        211,
        124,
        67,
        15,
        211,
        194,
        178,
        240
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "payer"
              }
            ]
          }
        },
        {
          "name": "laneZero",
          "writable": true
        },
        {
          "name": "laneOne",
          "writable": true
        },
        {
          "name": "laneTwo",
          "writable": true
        },
        {
          "name": "laneThree",
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "commitment",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    },
    {
      "name": "reveal",
      "discriminator": [
        9,
        35,
        59,
        190,
        167,
        249,
        76,
        115
      ],
      "accounts": [
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "secret",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    },
    {
      "name": "revokeSession",
      "discriminator": [
        86,
        92,
        198,
        120,
        144,
        2,
        7,
        194
      ],
      "accounts": [
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "epoch",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "sealRandomness",
      "discriminator": [
        217,
        137,
        207,
        224,
        46,
        127,
        20,
        156
      ],
      "accounts": [
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "settlePlayer",
      "discriminator": [
        163,
        18,
        55,
        160,
        79,
        20,
        108,
        114
      ],
      "accounts": [
        {
          "name": "keeper",
          "writable": true,
          "signer": true
        },
        {
          "name": "epoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "leaderboard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  101,
                  97,
                  100,
                  101,
                  114,
                  98,
                  111,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "player",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  108,
                  97,
                  121,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "player.owner",
                "account": "playerEpoch"
              }
            ]
          }
        },
        {
          "name": "laneZero",
          "writable": true
        },
        {
          "name": "laneOne",
          "writable": true
        },
        {
          "name": "laneTwo",
          "writable": true
        },
        {
          "name": "laneThree",
          "writable": true
        },
        {
          "name": "keeperCredit",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  101,
                  112,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "epoch.number",
                "account": "epoch"
              },
              {
                "kind": "account",
                "path": "keeper"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "sweepExpiredBonds",
      "discriminator": [
        1,
        249,
        168,
        169,
        133,
        122,
        53,
        171
      ],
      "accounts": [
        {
          "name": "sweeper",
          "signer": true
        },
        {
          "name": "protocol",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "oldEpoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "oldEpoch.number",
                "account": "epoch"
              }
            ]
          }
        },
        {
          "name": "currentEpoch",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  112,
                  111,
                  99,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "currentEpoch.number",
                "account": "epoch"
              }
            ]
          }
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "balanceLane",
      "discriminator": [
        86,
        99,
        74,
        32,
        136,
        235,
        161,
        16
      ]
    },
    {
      "name": "epoch",
      "discriminator": [
        93,
        83,
        120,
        89,
        151,
        138,
        152,
        108
      ]
    },
    {
      "name": "keeperCredit",
      "discriminator": [
        144,
        133,
        186,
        223,
        220,
        53,
        67,
        116
      ]
    },
    {
      "name": "leaderboard",
      "discriminator": [
        247,
        186,
        238,
        243,
        194,
        30,
        9,
        36
      ]
    },
    {
      "name": "playerEpoch",
      "discriminator": [
        137,
        171,
        173,
        56,
        206,
        74,
        94,
        250
      ]
    },
    {
      "name": "protocol",
      "discriminator": [
        45,
        39,
        101,
        43,
        115,
        72,
        131,
        40
      ]
    },
    {
      "name": "rivalry",
      "discriminator": [
        232,
        255,
        182,
        233,
        123,
        58,
        119,
        28
      ]
    }
  ],
  "events": [
    {
      "name": "absorbed",
      "discriminator": [
        249,
        137,
        89,
        153,
        248,
        170,
        218,
        32
      ]
    },
    {
      "name": "activePlayStarted",
      "discriminator": [
        170,
        57,
        19,
        174,
        150,
        172,
        163,
        76
      ]
    },
    {
      "name": "allocationClaimed",
      "discriminator": [
        21,
        221,
        147,
        215,
        183,
        47,
        37,
        188
      ]
    },
    {
      "name": "badgesRefreshed",
      "discriminator": [
        202,
        149,
        176,
        135,
        176,
        244,
        62,
        106
      ]
    },
    {
      "name": "bondClaimed",
      "discriminator": [
        4,
        151,
        48,
        253,
        112,
        78,
        238,
        101
      ]
    },
    {
      "name": "dumped",
      "discriminator": [
        31,
        216,
        22,
        251,
        198,
        136,
        83,
        46
      ]
    },
    {
      "name": "epochCancelled",
      "discriminator": [
        207,
        234,
        62,
        125,
        97,
        159,
        190,
        48
      ]
    },
    {
      "name": "epochCompleted",
      "discriminator": [
        23,
        26,
        1,
        148,
        161,
        59,
        181,
        142
      ]
    },
    {
      "name": "epochOpened",
      "discriminator": [
        136,
        166,
        130,
        170,
        78,
        145,
        67,
        78
      ]
    },
    {
      "name": "excessBondsSwept",
      "discriminator": [
        225,
        44,
        201,
        28,
        181,
        2,
        88,
        153
      ]
    },
    {
      "name": "keeperRewardClaimed",
      "discriminator": [
        174,
        189,
        160,
        163,
        251,
        30,
        23,
        52
      ]
    },
    {
      "name": "playerRegistered",
      "discriminator": [
        175,
        78,
        252,
        170,
        75,
        230,
        36,
        251
      ]
    },
    {
      "name": "playerRewardClaimed",
      "discriminator": [
        95,
        73,
        222,
        48,
        243,
        164,
        6,
        159
      ]
    },
    {
      "name": "playerSettled",
      "discriminator": [
        129,
        197,
        172,
        238,
        182,
        135,
        215,
        218
      ]
    },
    {
      "name": "protocolInitialized",
      "discriminator": [
        173,
        122,
        168,
        254,
        9,
        118,
        76,
        132
      ]
    },
    {
      "name": "randomnessSealed",
      "discriminator": [
        235,
        213,
        199,
        9,
        84,
        139,
        4,
        100
      ]
    },
    {
      "name": "redirectArmed",
      "discriminator": [
        71,
        169,
        239,
        80,
        165,
        4,
        252,
        231
      ]
    },
    {
      "name": "revealPhaseOpened",
      "discriminator": [
        225,
        48,
        78,
        116,
        171,
        223,
        118,
        66
      ]
    },
    {
      "name": "secretRevealed",
      "discriminator": [
        164,
        226,
        145,
        231,
        240,
        31,
        44,
        142
      ]
    },
    {
      "name": "sessionAuthorized",
      "discriminator": [
        62,
        191,
        225,
        155,
        148,
        108,
        182,
        4
      ]
    },
    {
      "name": "sessionRevoked",
      "discriminator": [
        90,
        48,
        35,
        234,
        203,
        192,
        126,
        211
      ]
    },
    {
      "name": "settlementStarted",
      "discriminator": [
        131,
        24,
        2,
        55,
        117,
        102,
        209,
        156
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "wrongPhase",
      "msg": "This instruction is not available in the current phase"
    },
    {
      "code": 6001,
      "name": "tooEarly",
      "msg": "The current phase has not reached its transition time"
    },
    {
      "code": 6002,
      "name": "windowClosed",
      "msg": "The current window has closed"
    },
    {
      "code": 6003,
      "name": "underfilledEpoch",
      "msg": "The epoch does not have enough registered players"
    },
    {
      "code": 6004,
      "name": "insufficientReveals",
      "msg": "The epoch does not have enough valid randomness reveals"
    },
    {
      "code": 6005,
      "name": "participantCapReached",
      "msg": "The epoch participant cap has been reached"
    },
    {
      "code": 6006,
      "name": "invalidCommitment",
      "msg": "The supplied commitment is invalid"
    },
    {
      "code": 6007,
      "name": "alreadyRevealed",
      "msg": "The player has already revealed"
    },
    {
      "code": 6008,
      "name": "allocationAlreadyClaimed",
      "msg": "The allocation has already been claimed"
    },
    {
      "code": 6009,
      "name": "allocationNotClaimed",
      "msg": "The player must claim an allocation first"
    },
    {
      "code": 6010,
      "name": "unauthorized",
      "msg": "The signer is not authorized for this player"
    },
    {
      "code": 6011,
      "name": "selfAction",
      "msg": "Self-targeted actions are not allowed"
    },
    {
      "code": 6012,
      "name": "invalidAmount",
      "msg": "The action amount is invalid"
    },
    {
      "code": 6013,
      "name": "wrongTargetLane",
      "msg": "The target lane is not the deterministic lane for this action"
    },
    {
      "code": 6014,
      "name": "insufficientSpendableDump",
      "msg": "The source lane does not have enough unlocked DUMP"
    },
    {
      "code": 6015,
      "name": "heatCapacityExceeded",
      "msg": "This action would exceed the lane's action heat capacity"
    },
    {
      "code": 6016,
      "name": "redirectNotReady",
      "msg": "REDIRECT cannot be armed yet"
    },
    {
      "code": 6017,
      "name": "noGuard",
      "msg": "REDIRECT requires non-zero Guard"
    },
    {
      "code": 6018,
      "name": "alreadySettled",
      "msg": "The player has already been settled"
    },
    {
      "code": 6019,
      "name": "settlementIncomplete",
      "msg": "Not every participant has been settled"
    },
    {
      "code": 6020,
      "name": "notWinner",
      "msg": "The player is not an epoch winner"
    },
    {
      "code": 6021,
      "name": "rewardAlreadyClaimed",
      "msg": "This reward has already been claimed"
    },
    {
      "code": 6022,
      "name": "bondAlreadyClaimed",
      "msg": "This bond has already been claimed"
    },
    {
      "code": 6023,
      "name": "bondForfeited",
      "msg": "The player did not qualify for a bond refund"
    },
    {
      "code": 6024,
      "name": "insufficientEpochFunds",
      "msg": "The epoch account cannot fund the expected deterministic payout"
    },
    {
      "code": 6025,
      "name": "invalidNextEpoch",
      "msg": "The supplied epoch number is not the next epoch"
    },
    {
      "code": 6026,
      "name": "invalidSession",
      "msg": "The session duration or action allowance is invalid"
    },
    {
      "code": 6027,
      "name": "invalidRivalry",
      "msg": "The rivalry account does not match this actor and target"
    },
    {
      "code": 6028,
      "name": "cleanupNotReady",
      "msg": "The account cannot be reclaimed until its bond and reward are resolved"
    },
    {
      "code": 6029,
      "name": "pendingReward",
      "msg": "The player still has an unclaimed GLORY reward"
    },
    {
      "code": 6030,
      "name": "arithmeticOverflow",
      "msg": "An arithmetic operation overflowed"
    }
  ],
  "types": [
    {
      "name": "absorbed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "actor",
            "type": "pubkey"
          },
          {
            "name": "target",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "destinationLane",
            "type": "u8"
          },
          {
            "name": "sourceLane",
            "type": "u8"
          },
          {
            "name": "guardAfter",
            "type": "u64"
          },
          {
            "name": "lockedUntil",
            "type": "i64"
          },
          {
            "name": "absorbHeat",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "activePlayStarted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "endsAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "allocationClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "revealed",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "badgesRefreshed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "badges",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "balanceLane",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "index",
            "type": "u8"
          },
          {
            "name": "balance",
            "type": "u64"
          },
          {
            "name": "guard",
            "type": "u64"
          },
          {
            "name": "lockedAmount",
            "type": "u64"
          },
          {
            "name": "lockedUntil",
            "type": "i64"
          },
          {
            "name": "redirectArmed",
            "type": "bool"
          },
          {
            "name": "redirectReadyAt",
            "type": "i64"
          },
          {
            "name": "redirectedVolume",
            "type": "u64"
          },
          {
            "name": "cumulativeWeighted",
            "type": "u128"
          },
          {
            "name": "lastCheckpointAt",
            "type": "i64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "bondClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "dumped",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "actor",
            "type": "pubkey"
          },
          {
            "name": "target",
            "type": "pubkey"
          },
          {
            "name": "attempted",
            "type": "u64"
          },
          {
            "name": "landed",
            "type": "u64"
          },
          {
            "name": "redirected",
            "type": "u64"
          },
          {
            "name": "sourceLane",
            "type": "u8"
          },
          {
            "name": "targetLane",
            "type": "u8"
          },
          {
            "name": "dumpHeat",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "epoch",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "number",
            "type": "u64"
          },
          {
            "name": "phase",
            "type": {
              "defined": {
                "name": "phase"
              }
            }
          },
          {
            "name": "registrationStartedAt",
            "type": "i64"
          },
          {
            "name": "registrationEndsAt",
            "type": "i64"
          },
          {
            "name": "revealEndsAt",
            "type": "i64"
          },
          {
            "name": "activeStartsAt",
            "type": "i64"
          },
          {
            "name": "activeEndsAt",
            "type": "i64"
          },
          {
            "name": "entropy",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "seed",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "participantCount",
            "type": "u32"
          },
          {
            "name": "revealedCount",
            "type": "u32"
          },
          {
            "name": "settledCount",
            "type": "u32"
          },
          {
            "name": "eligibleCount",
            "type": "u32"
          },
          {
            "name": "winnerCount",
            "type": "u16"
          },
          {
            "name": "totalRewardPool",
            "type": "u64"
          },
          {
            "name": "playerRewardPool",
            "type": "u64"
          },
          {
            "name": "keeperRewardPool",
            "type": "u64"
          },
          {
            "name": "worstPlayer",
            "type": "pubkey"
          },
          {
            "name": "worstScore",
            "type": "u64"
          },
          {
            "name": "bondsCollected",
            "type": "u64"
          },
          {
            "name": "bondRefundsPaid",
            "type": "u64"
          },
          {
            "name": "settlementBountiesPaid",
            "type": "u64"
          },
          {
            "name": "bondClaimEndsAt",
            "type": "i64"
          },
          {
            "name": "sweptExcessBonds",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "epochCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "participantCount",
            "type": "u32"
          },
          {
            "name": "revealedCount",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "epochCompleted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "eligibleCount",
            "type": "u32"
          },
          {
            "name": "winnerCount",
            "type": "u16"
          },
          {
            "name": "totalRewardPool",
            "type": "u64"
          },
          {
            "name": "playerRewardPool",
            "type": "u64"
          },
          {
            "name": "keeperRewardPool",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "epochOpened",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "registrationEndsAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "excessBondsSwept",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "fromEpoch",
            "type": "u64"
          },
          {
            "name": "toEpoch",
            "type": "u64"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "heatState",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "units",
            "type": "u32"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "keeperCredit",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "keeper",
            "type": "pubkey"
          },
          {
            "name": "playersSettled",
            "type": "u32"
          },
          {
            "name": "claimed",
            "type": "bool"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "keeperRewardClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "keeper",
            "type": "pubkey"
          },
          {
            "name": "playersSettled",
            "type": "u32"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "leaderboard",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "entries",
            "type": {
              "vec": {
                "defined": {
                  "name": "winnerEntry"
                }
              }
            }
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "phase",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "registration"
          },
          {
            "name": "reveal"
          },
          {
            "name": "planning"
          },
          {
            "name": "active"
          },
          {
            "name": "settling"
          },
          {
            "name": "complete"
          },
          {
            "name": "cancelled"
          }
        ]
      }
    },
    {
      "name": "playerEpoch",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "commitment",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "revealed",
            "type": "bool"
          },
          {
            "name": "allocationClaimed",
            "type": "bool"
          },
          {
            "name": "settled",
            "type": "bool"
          },
          {
            "name": "bondClaimed",
            "type": "bool"
          },
          {
            "name": "startingAllocation",
            "type": "u64"
          },
          {
            "name": "finalScore",
            "type": "u64"
          },
          {
            "name": "dumpHeat",
            "type": {
              "defined": {
                "name": "heatState"
              }
            }
          },
          {
            "name": "absorbHeat",
            "type": {
              "defined": {
                "name": "heatState"
              }
            }
          },
          {
            "name": "impactPpm",
            "type": "u64"
          },
          {
            "name": "lateImpactPpm",
            "type": "u64"
          },
          {
            "name": "meaningfulActions",
            "type": "u32"
          },
          {
            "name": "distinctOpponents",
            "type": "u16"
          },
          {
            "name": "sessionDelegate",
            "type": "pubkey"
          },
          {
            "name": "sessionExpiresAt",
            "type": "i64"
          },
          {
            "name": "sessionActionsRemaining",
            "type": "u16"
          },
          {
            "name": "badges",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "playerRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "playerRewardClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "rank",
            "type": "u16"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "playerSettled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "score",
            "type": "u64"
          },
          {
            "name": "eligible",
            "type": "bool"
          },
          {
            "name": "keeper",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "protocol",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u16"
          },
          {
            "name": "currentEpoch",
            "type": "u64"
          },
          {
            "name": "gloryMint",
            "type": "pubkey"
          },
          {
            "name": "gloryCommitted",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "protocolInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "protocol",
            "type": "pubkey"
          },
          {
            "name": "gloryMint",
            "type": "pubkey"
          },
          {
            "name": "firstEpoch",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "randomnessSealed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "seed",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "activeStartsAt",
            "type": "i64"
          },
          {
            "name": "activeEndsAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "redirectArmed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "lane",
            "type": "u8"
          },
          {
            "name": "guard",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "revealPhaseOpened",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "revealEndsAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "rivalry",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "actor",
            "type": "pubkey"
          },
          {
            "name": "target",
            "type": "pubkey"
          },
          {
            "name": "rentPayer",
            "type": "pubkey"
          },
          {
            "name": "impactCreditedPpm",
            "type": "u64"
          },
          {
            "name": "actionCount",
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "secretRevealed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "sessionAuthorized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "delegate",
            "type": "pubkey"
          },
          {
            "name": "expiresAt",
            "type": "i64"
          },
          {
            "name": "actions",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "sessionRevoked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "player",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "settlementStarted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "epoch",
            "type": "u64"
          },
          {
            "name": "participantCount",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "winnerEntry",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "player",
            "type": "pubkey"
          },
          {
            "name": "score",
            "type": "u64"
          },
          {
            "name": "impactPpm",
            "type": "u64"
          },
          {
            "name": "distinctOpponents",
            "type": "u16"
          },
          {
            "name": "tieBreaker",
            "type": "u64"
          },
          {
            "name": "claimed",
            "type": "bool"
          }
        ]
      }
    }
  ]
};
