const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("DUMP/GLORY Token System", function () {
  let dumpToken, gloryToken, feePot, bridgeGatekeeper;
  let owner, user1, user2, user3;
  let initialSupply;

  beforeEach(async function () {
    [owner, user1, user2, user3] = await ethers.getSigners();
    initialSupply = ethers.parseEther("1000000"); // 1M DUMP

    // Deploy DUMP token
    const DumpToken = await ethers.getContractFactory("DumpToken");
    dumpToken = await DumpToken.deploy("DUMP Token", "DUMP", initialSupply);

    // Deploy GLORY token
    const GloryToken = await ethers.getContractFactory("GloryToken");
    gloryToken = await GloryToken.deploy("GLORY Token", "GLORY", await dumpToken.getAddress());

    // Deploy FeePot with placeholder addresses
    const FeePot = await ethers.getContractFactory("FeePot");
    feePot = await FeePot.deploy(
      await dumpToken.getAddress(),
      await gloryToken.getAddress(),
      "0x4200000000000000000000000000000000000006", // WETH
      "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC
      "0x8909Dc15e40173Ff4699343b6eB8132c65e18eC6", // Factory
      "0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24"  // Router
    );

    // Deploy BridgeGatekeeper
    const BridgeGatekeeper = await ethers.getContractFactory("BridgeGatekeeper");
    bridgeGatekeeper = await BridgeGatekeeper.deploy(
      await dumpToken.getAddress(),
      "0x0000000000000000000000000000000000000000"
    );
  });

  describe("DUMP Token", function () {
    it("Should have correct initial supply", async function () {
      expect(await dumpToken.totalSupply()).to.equal(initialSupply);
      expect(await dumpToken.balanceOf(owner.address)).to.equal(initialSupply);
    });

    it("Should require staking for participation", async function () {
      const minStake = await dumpToken.getMinimumStake();
      expect(minStake).to.be.gt(0);

      // Try to transfer without staking
      await expect(
        dumpToken.transfer(user1.address, ethers.parseEther("100"))
      ).to.be.revertedWith("Must be active participant");
    });

    it("Should allow staking for participation", async function () {
      const minStake = await dumpToken.getMinimumStake();
      
      // Stake for participation
      await dumpToken.stakeForParticipation(minStake);
      
      expect(await dumpToken.isActiveParticipant(owner.address)).to.be.true;
      expect(await dumpToken.stakedAmount(owner.address)).to.equal(minStake);
    });

    it("Should apply demurrage correctly", async function () {
      const minStake = await dumpToken.getMinimumStake();
      await dumpToken.stakeForParticipation(minStake);
      
      const initialBalance = await dumpToken.balanceOf(owner.address);
      
      // Fast forward 1 day
      await ethers.provider.send("evm_increaseTime", [86400]); // 1 day
      await ethers.provider.send("evm_mine");
      
      // Apply demurrage
      await dumpToken.applyDemurrage(owner.address);
      
      const newBalance = await dumpToken.balanceOf(owner.address);
      expect(newBalance).to.be.lt(initialBalance);
    });

    it("Should calculate cooldowns based on transfer amount", async function () {
      const minStake = await dumpToken.getMinimumStake();
      await dumpToken.stakeForParticipation(minStake);
      
      const smallAmount = ethers.parseEther("100");
      const largeAmount = ethers.parseEther("100000");
      
      const smallCooldown = await dumpToken.computeCooldown(smallAmount);
      const largeCooldown = await dumpToken.computeCooldown(largeAmount);
      
      expect(largeCooldown).to.be.gt(smallCooldown);
    });

    it("Should collect transfer fees", async function () {
      const minStake = await dumpToken.getMinimumStake();
      await dumpToken.stakeForParticipation(minStake);
      
      const transferAmount = ethers.parseEther("1000");
      const initialFeePot = await dumpToken.getFeePot();
      
      await dumpToken.transfer(user1.address, transferAmount);
      
      const newFeePot = await dumpToken.getFeePot();
      expect(newFeePot).to.be.gt(initialFeePot);
    });
  });

  describe("GLORY Token", function () {
    it("Should have correct initial supply", async function () {
      const totalSupply = await gloryToken.totalSupply();
      expect(totalSupply).to.equal(ethers.parseEther("1000000")); // 1M GLORY
    });

    it("Should reserve bug bounty", async function () {
      const bugBounty = await gloryToken.bugBountyReserve();
      expect(bugBounty).to.be.gt(0);
    });

    it("Should track epoch data", async function () {
      const currentEpoch = await gloryToken.currentEpoch();
      expect(currentEpoch).to.equal(1);
    });
  });

  describe("FeePot", function () {
    it("Should track fee collection", async function () {
      const initialFees = await feePot.totalFeesCollected();
      expect(initialFees).to.equal(0);
    });

    it("Should have circuit breaker", async function () {
      expect(await feePot.emergencyPaused()).to.be.false;
    });
  });

  describe("BridgeGatekeeper", function () {
    it("Should enforce cooldowns", async function () {
      const canTransfer = await bridgeGatekeeper.canTransfer(user1.address, ethers.parseEther("1000"));
      expect(canTransfer).to.be.true; // Should be true initially
    });

    it("Should track epoch transfers", async function () {
      const epochStats = await bridgeGatekeeper.getEpochTransferStats(1);
      expect(epochStats).to.equal(0);
    });
  });

  describe("Integration", function () {
    it("Should allow complete game flow", async function () {
      // 1. Users stake for participation
      const minStake = await dumpToken.getMinimumStake();
      await dumpToken.stakeForParticipation(minStake);
      await dumpToken.connect(user1).stakeForParticipation(minStake);
      await dumpToken.connect(user2).stakeForParticipation(minStake);
      
      // 2. Users transfer DUMP (simulating the game)
      await dumpToken.transfer(user1.address, ethers.parseEther("1000"));
      await dumpToken.connect(user1).transfer(user2.address, ethers.parseEther("500"));
      await dumpToken.connect(user2).transfer(user3.address, ethers.parseEther("200"));
      
      // 3. Check that fees were collected
      const feePot = await dumpToken.getFeePot();
      expect(feePot).to.be.gt(0);
      
      // 4. Check that cooldowns are active
      const cooldownEnd = await dumpToken.cooldownEndTime(owner.address);
      expect(cooldownEnd).to.be.gt(0);
    });
  });
});