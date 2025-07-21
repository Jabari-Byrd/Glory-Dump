require("@nomicfoundation/hardhat-toolbox");

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.24",  // Latest with all security fixes
    settings: {
      optimizer: {
        enabled: true,
        runs: 500,      // Higher for your game's frequency
        details: {
          yul: true,    // Enable Yul optimizer
          yulDetails: {
            optimizerSteps: "u",  // Ultra optimization
            stackAllocation: true // Better stack management
          }
        }
      }
    },
    viaIR: true,      // Enable intermediate representation
    outputSelection: {
      "*": {
        "*": [
          "abi",
          "evm.bytecode",
          "evm.deployedBytecode",
          "evm.methodIdentifiers"
        ]
      }
    }
  },
  networks: {
    hardhat: {
      chainId: 31337
    },
    "base-testnet": {
      url: "https://goerli.base.org",
      chainId: 84531,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    "base": {
      url: "https://mainnet.base.org",
      chainId: 8453,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    }
  },
  etherscan: {
    apiKey: {
      "base": process.env.BASESCAN_API_KEY || "",
      "base-testnet": process.env.BASESCAN_API_KEY || ""
    },
    customChains: [
      {
        network: "base",
        chainId: 8453,
        urls: {
          apiURL: "https://api.basescan.org/api",
          browserURL: "https://basescan.org"
        }
      },
      {
        network: "base-testnet",
        chainId: 84531,
        urls: {
          apiURL: "https://api-goerli.basescan.org/api",
          browserURL: "https://goerli.basescan.org"
        }
      }
    ]
  }
};