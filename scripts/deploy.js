const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with account:", deployer.address);

  // Deploy DUMP token
  console.log("Deploying DUMP token...");
  const DumpToken = await ethers.getContractFactory("DumpToken");
  const dumpToken = await DumpToken.deploy(
    "DUMP Token",
    "DUMP",
    ethers.parseEther("1000000") // 1M initial supply
  );
  await dumpToken.waitForDeployment();
  console.log("DUMP token deployed to:", await dumpToken.getAddress());

  // Deploy GLORY token
  console.log("Deploying GLORY token...");
  const GloryToken = await ethers.getContractFactory("GloryToken");
  const gloryToken = await GloryToken.deploy(
    "GLORY Token",
    "GLORY",
    await dumpToken.getAddress()
  );
  await gloryToken.waitForDeployment();
  console.log("GLORY token deployed to:", await gloryToken.getAddress());

  // Deploy FeePot (with placeholder addresses for now)
  console.log("Deploying FeePot...");
  const FeePot = await ethers.getContractFactory("FeePot");
  const feePot = await FeePot.deploy(
    await dumpToken.getAddress(),
    await gloryToken.getAddress(),
    "0x4200000000000000000000000000000000000006", // WETH on Base
    "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC on Base
    "0x8909Dc15e40173Ff4699343b6eB8132c65e18eC6", // Uniswap V2 Factory on Base
    "0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24"  // Uniswap V2 Router on Base
  );
  await feePot.waitForDeployment();
  console.log("FeePot deployed to:", await feePot.getAddress());

  // Deploy BridgeGatekeeper (with placeholder bridge address)
  console.log("Deploying BridgeGatekeeper...");
  const BridgeGatekeeper = await ethers.getContractFactory("BridgeGatekeeper");
  const bridgeGatekeeper = await BridgeGatekeeper.deploy(
    await dumpToken.getAddress(),
    "0x0000000000000000000000000000000000000000" // Placeholder bridge address
  );
  await bridgeGatekeeper.waitForDeployment();
  console.log("BridgeGatekeeper deployed to:", await bridgeGatekeeper.getAddress());

  // Set up permissions and relationships
  console.log("Setting up contract relationships...");
  
  // TODO: Set bridge address in DumpToken
  // TODO: Set fee pot address in DumpToken
  // TODO: Set bridge gatekeeper permissions

  console.log("Deployment complete!");
  console.log("=== Contract Addresses ===");
  console.log("DUMP Token:", await dumpToken.getAddress());
  console.log("GLORY Token:", await gloryToken.getAddress());
  console.log("FeePot:", await feePot.getAddress());
  console.log("BridgeGatekeeper:", await bridgeGatekeeper.getAddress());
  console.log("==========================");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });