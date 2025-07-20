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
    });

    it("Should allow theft with proper constraints", async function () {
      const minStake = await dumpToken.getMinimumStake();

      // Setup both users as active participants
      await dumpToken.stakeForParticipation(minStake);
      await dumpToken.transfer(user1.address, ethers.parseEther("10000"));
      await dumpToken.connect(user1).stakeForParticipation(minStake);

      const theftAmount = ethers.parseEther("1000");
      const victimBalance = await dumpToken.balanceOf(user1.address);

      // Execute theft
      await dumpToken.stealDump(user1.address, theftAmount);

      // Check victim lost tokens
      const newVictimBalance = await dumpToken.balanceOf(user1.address);
      expect(newVictimBalance).to.be.lt(victimBalance);

      // Check thief gained tokens (minus fees and costs)
      const thiefBalance = await dumpToken.balanceOf(owner.address);
      expect(thiefBalance).to.be.gt(0);
    });

    it("Should enforce theft cooldowns", async function () {
      const minStake = await dumpToken.getMinimumStake();

      // Setup both users as active participants
      await dumpToken.stakeForParticipation(minStake);
      await dumpToken.transfer(user1.address, ethers.parseEther("10000"));
      await dumpToken.connect(user1).stakeForParticipation(minStake);

      const theftAmount = ethers.parseEther("1000");

      // First theft should succeed
      await dumpToken.stealDump(user1.address, theftAmount);

      // Second theft should fail due to cooldown
      await expect(
        dumpToken.stealDump(user1.address, theftAmount)
      ).to.be.revertedWith("Theft cooldown active");
    });

        it("Should calculate epoch-weighted theft costs with continuous scaling", async function () {
      const minStake = await dumpToken.getMinimumStake();
      await dumpToken.stakeForParticipation(minStake);
      
      const theftAmount = ethers.parseEther("1000");
      
      // Day 1 cost (should be lowest)
      const costDay1 = await dumpToken.calculateTheftCost(theftAmount);
      
      // Week 2 cost (should be higher)
      await ethers.provider.send("evm_increaseTime", [14 * 24 * 3600]); // 14 days
      await ethers.provider.send("evm_mine");
      const costWeek2 = await dumpToken.calculateTheftCost(theftAmount);
      expect(costWeek2).to.be.gt(costDay1);
      
      // Last week cost (should be even higher)
      await ethers.provider.send("evm_increaseTime", [7 * 24 * 3600]); // 7 more days
      await ethers.provider.send("evm_mine");
      const costLastWeek = await dumpToken.calculateTheftCost(theftAmount);
      expect(costLastWeek).to.be.gt(costWeek2);
      
      // Last day cost (should be much higher)
      await ethers.provider.send("evm_increaseTime", [6 * 24 * 3600]); // 6 more days
      await ethers.provider.send("evm_mine");
      const costLastDay = await dumpToken.calculateTheftCost(theftAmount);
      expect(costLastDay).to.be.gt(costLastWeek);
      
      // Last hour cost (should be exponential)
      await ethers.provider.send("evm_increaseTime", [23 * 3600]); // 23 more hours
      await ethers.provider.send("evm_mine");
      const costLastHour = await dumpToken.calculateTheftCost(theftAmount);
      expect(costLastHour).to.be.gt(costLastDay);
      
      // Last minute cost (should be meme territory)
      await ethers.provider.send("evm_increaseTime", [59 * 60]); // 59 more minutes
      await ethers.provider.send("evm_mine");
      const costLastMinute = await dumpToken.calculateTheftCost(theftAmount);
      expect(costLastMinute).to.be.gt(costLastHour);
    });
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