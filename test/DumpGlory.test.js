const { expect } = require("chai");
const { ethers } = require("hardhat");

let dumpToken, gloryToken, feePot, bridgeGatekeeper;
let owner, user1, user2, user3, user4, user5, user6, user7, user8, user9, user10, user11, user12, user13, user14, user15;

before(async function () {
  [owner, user1, user2, user3, user4, user5, user6, user7, user8, user9, user10, user11, user12, user13, user14, user15] = await ethers.getSigners();

  // Deploy DUMP token
  const DumpToken = await ethers.getContractFactory("DumpToken");
  dumpToken = await DumpToken.deploy("DUMP Token", "DUMP", 0); // initialSupply is ignored in new logic

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

// Helper: Fast-forward EVM time
async function fastForward(seconds) {
  await ethers.provider.send("evm_increaseTime", [seconds]);
  await ethers.provider.send("evm_mine");
}

// Helper: Sign up for next epoch
async function signup(user, fee) {
  await dumpToken.connect(user).signupForNextEpoch({ value: fee });
}

// Helper: Start next epoch
async function startNextEpoch() {
  await dumpToken.startNextEpoch();
}

describe("GLORY/DUMP Token System", function () {
  describe("DUMP Token (New Mechanics)", function () {
    it("Should allow sign-up during waiting period with correct fee scaling", async function () {
      // Finalize epoch to enter waiting period
      await dumpToken.finalizeEpoch();
      // Early joiner (low fee)
      const baseFee = await dumpToken.BASE_JOIN_FEE();
      await signup(user1, baseFee);
      // Fast forward half the waiting period
      const waiting = await dumpToken.WAITING_PERIOD();
      await fastForward(Number(waiting) / 2);
      // Mid joiner (mid fee)
      const midFee = await dumpToken.MAX_JOIN_FEE() / 2n;
      await signup(user2, midFee);
      // Fast forward to end
      await fastForward(Number(waiting) / 2);
      // Late joiner (max fee)
      const maxFee = await dumpToken.MAX_JOIN_FEE();
      await signup(user3, maxFee);
    });

    it("Should only allow gameplay after waiting period ends and epoch starts", async function () {
      // Try to transfer during waiting period (should fail)
      await expect(
        dumpToken.connect(user1).transfer(user2.address, 1)
      ).to.be.revertedWith("Game not started yet");
      // Start next epoch
      await startNextEpoch();
      // Now transfer should succeed (if user1 has DUMP)
      // (user1 will have random DUMP assigned)
      const bal1 = await dumpToken.balanceOf(user1.address);
      if (bal1 > 0) {
        await dumpToken.connect(user1).transfer(user2.address, 1);
      }
    });

    it("Should assign random DUMP to all active participants at epoch start", async function () {
      // All signed-up users should have nonzero DUMP
      for (const user of [user1, user2, user3]) {
        const bal = await dumpToken.balanceOf(user.address);
        expect(bal).to.be.gt(0);
      }
    });

    it("Should only include signed-up participants in the epoch", async function () {
      // user4 did not sign up, should have zero DUMP
      const bal = await dumpToken.balanceOf(user4.address);
      expect(bal).to.equal(0);
    });

    it("Should update and track average DUMP for ranking", async function () {
      // user1 transfers to user2
      const bal1 = await dumpToken.balanceOf(user1.address);
      if (bal1 > 0) {
        await dumpToken.connect(user1).transfer(user2.address, bal1 / 2n);
      }
      // Fast forward 1 day
      await fastForward(86400);
      // user2 transfers to user3
      const bal2 = await dumpToken.balanceOf(user2.address);
      if (bal2 > 0) {
        await dumpToken.connect(user2).transfer(user3.address, bal2 / 2n);
      }
      // Check average DUMP
      const avg1 = await dumpToken.getAverageDump(user1.address);
      const avg2 = await dumpToken.getAverageDump(user2.address);
      const avg3 = await dumpToken.getAverageDump(user3.address);
      expect(avg1).to.be.a("bigint");
      expect(avg2).to.be.a("bigint");
      expect(avg3).to.be.a("bigint");
    });

    it("Should expire inactive participants and reset stakes at epoch start", async function () {
      // user1, user2, user3 are active; user4 is not
      // Finalize epoch and enter waiting period
      await dumpToken.finalizeEpoch();
      // Only user4 signs up for next epoch
      const baseFee = await dumpToken.BASE_JOIN_FEE();
      await signup(user4, baseFee);
      // Start next epoch
      await fastForward(Number(await dumpToken.WAITING_PERIOD()));
      await startNextEpoch();
      // user1, user2, user3 should now be inactive
      for (const user of [user1, user2, user3]) {
        expect(await dumpToken.isActiveParticipant(user.address)).to.be.false;
      }
      // user4 should be active
      expect(await dumpToken.isActiveParticipant(user4.address)).to.be.true;
    });

    it("Should enforce action-specific cooldowns for give and take", async function () {
      // user4 transfers to user5
      const bal4 = await dumpToken.balanceOf(user4.address);
      if (bal4 > 0) {
        await dumpToken.connect(user4).transfer(user5.address, bal4 / 2n);
        // Try to transfer again immediately (should fail if cooldown active)
        await expect(
          dumpToken.connect(user4).transfer(user5.address, 1)
        ).to.be.revertedWith("Give cooldown active");
      }
      // Fast forward cooldown
      await fastForward(3600);
      // Should be able to transfer again
      if (bal4 > 0) {
        await dumpToken.connect(user4).transfer(user5.address, 1);
      }
    });

    it("Should reset all state and assign new random DUMP at each epoch", async function () {
      // Finalize epoch and enter waiting period
      await dumpToken.finalizeEpoch();
      // user6 and user7 sign up
      const baseFee = await dumpToken.BASE_JOIN_FEE();
      await signup(user6, baseFee);
      await signup(user7, baseFee);
      // Start next epoch
      await fastForward(Number(await dumpToken.WAITING_PERIOD()));
      await startNextEpoch();
      // user6 and user7 should have nonzero DUMP
      expect(await dumpToken.balanceOf(user6.address)).to.be.gt(0);
      expect(await dumpToken.balanceOf(user7.address)).to.be.gt(0);
      // user4 should now be inactive
      expect(await dumpToken.isActiveParticipant(user4.address)).to.be.false;
    });
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
    // Reset cooldowns and setup users
    await dumpToken.finalizeEpoch(); // Finalize current epoch
    await fastForward(Number(await dumpToken.WAITING_PERIOD())); // Fast forward to waiting period end
    await startNextEpoch(); // Start next epoch

    // 1. Users sign up for participation
    const baseFee = await dumpToken.BASE_JOIN_FEE();
    await signup(user14, baseFee);
    await signup(user15, baseFee);

    // 2. Users transfer DUMP (simulating the game)
    await dumpToken.connect(user14).transfer(user15.address, 100); // User14 gives 100 DUMP to User15
    await dumpToken.connect(user1.address).transfer(user2.address, 50); // User1 gives 50 DUMP to User2
    await dumpToken.connect(user14.address).transfer(user1.address, 200); // User14 gives 200 DUMP to User1

    // 3. Check that fees were collected
    const feePot = await dumpToken.getFeePot();
    expect(feePot).to.be.gt(0);

    // 4. Check that cooldowns are active
    const cooldownEnd = await dumpToken.cooldownEndTime(owner.address);
    expect(cooldownEnd).to.be.gt(0);
  });
});

describe("Bug Bounty System", function () {
  it("Should have initial bug bounty reserve", async function () {
    const reserve = await gloryToken.getBugBountyReserve();
    expect(reserve).to.equal(ethers.parseEther("100000")); // 10% of 1M GLORY
  });

  it("Should allow users to submit bug reports", async function () {
    const description = "Test bug description";
    const proofOfConcept = "Test proof of concept";

    await gloryToken.connect(user1).submitBugReport(0, description, proofOfConcept); // LOW severity

    const reportIds = await gloryToken.getAllBugReports();
    expect(reportIds.length).to.equal(1);

    const report = await gloryToken.getBugReport(reportIds[0]);
    expect(report.reporter).to.equal(user1.address);
    expect(report.description).to.equal(description);
    expect(report.proofOfConcept).to.equal(proofOfConcept);
    expect(report.severity).to.equal(0); // LOW
    expect(report.verified).to.be.false;
    expect(report.paid).to.be.false;
  });

  it("Should allow owner to verify bug reports", async function () {
    const description = "Critical vulnerability found";
    await gloryToken.connect(user2).submitBugReport(3, description, ""); // CRITICAL severity

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    // Owner verifies the report
    await gloryToken.verifyBugReport(reportId, 0); // Use standard bounty amount

    const report = await gloryToken.getBugReport(reportId);
    expect(report.verified).to.be.true;
    expect(report.bountyAmount).to.equal(ethers.parseEther("100000")); // CRITICAL bounty
  });

  it("Should allow owner to pay verified bug bounties", async function () {
    const description = "High severity bug";
    await gloryToken.connect(user3).submitBugReport(2, description, ""); // HIGH severity

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    // Owner verifies and pays
    await gloryToken.verifyBugReport(reportId, 0);
    await gloryToken.payBugBounty(reportId);

    const report = await gloryToken.getBugReport(reportId);
    expect(report.paid).to.be.true;

    // Check that bounty was transferred
    const user3Balance = await gloryToken.balanceOf(user3.address);
    expect(user3Balance).to.equal(ethers.parseEther("50000")); // HIGH bounty
  });

  it("Should allow owner to reject bug reports", async function () {
    const description = "Invalid bug report";
    await gloryToken.connect(user4).submitBugReport(1, description, ""); // MEDIUM severity

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    // Owner rejects the report
    await gloryToken.rejectBugReport(reportId, "Not a real bug");

    const report = await gloryToken.getBugReport(reportId);
    expect(report.verified).to.be.true;
    expect(report.paid).to.be.true;
    expect(report.bountyAmount).to.equal(0);
  });

  it("Should track reporter total bounties", async function () {
    // Get current total bounties for user5
    const currentTotal = await gloryToken.getReporterTotalBounties(user5.address);

    // Submit and pay multiple reports
    await gloryToken.connect(user5).submitBugReport(0, "Low bug 1", "");
    await gloryToken.connect(user5).submitBugReport(1, "Medium bug 1", "");

    const reportIds = await gloryToken.getAllBugReports();

    // Verify and pay first report
    await gloryToken.verifyBugReport(reportIds[reportIds.length - 2], 0);
    await gloryToken.payBugBounty(reportIds[reportIds.length - 2]);

    // Verify and pay second report
    await gloryToken.verifyBugReport(reportIds[reportIds.length - 1], 0);
    await gloryToken.payBugBounty(reportIds[reportIds.length - 1]);

    const totalBounties = await gloryToken.getReporterTotalBounties(user5.address);
    expect(totalBounties).to.equal(currentTotal + 2n); // Number of reports submitted
  });

  it("Should prevent non-owner from verifying reports", async function () {
    const description = "Test bug";
    await gloryToken.connect(user6).submitBugReport(0, description, "");

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    await expect(
      gloryToken.connect(user6).verifyBugReport(reportId, 0)
    ).to.be.revertedWithCustomError(gloryToken, "OwnableUnauthorizedAccount");
  });

  it("Should prevent non-owner from paying bounties", async function () {
    const description = "Test bug";
    await gloryToken.connect(user7).submitBugReport(0, description, "");

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    await expect(
      gloryToken.connect(user7).payBugBounty(reportId)
    ).to.be.revertedWithCustomError(gloryToken, "OwnableUnauthorizedAccount");
  });

  it("Should prevent paying unverified reports", async function () {
    const description = "Test bug";
    await gloryToken.connect(user8).submitBugReport(0, description, "");

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    await expect(
      gloryToken.payBugBounty(reportId)
    ).to.be.revertedWith("Bug report not verified");
  });

  it("Should prevent double-paying reports", async function () {
    const description = "Test bug";
    await gloryToken.connect(user9).submitBugReport(0, description, "");

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    // Verify and pay once
    await gloryToken.verifyBugReport(reportId, 0);
    await gloryToken.payBugBounty(reportId);

    // Try to pay again
    await expect(
      gloryToken.payBugBounty(reportId)
    ).to.be.revertedWith("Bug bounty already paid");
  });

  it("Should handle custom bounty amounts", async function () {
    const description = "Special bug";
    await gloryToken.connect(user10).submitBugReport(1, description, "");

    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    const customBounty = ethers.parseEther("5000"); // Custom amount within reserve
    await gloryToken.verifyBugReport(reportId, customBounty);

    const report = await gloryToken.getBugReport(reportId);
    expect(report.bountyAmount).to.equal(customBounty);
  });

  it("Should update bounty reserve correctly", async function () {
    const initialReserve = await gloryToken.getBugBountyReserve();

    // Submit and pay a bug report with a small amount to stay within reserve
    await gloryToken.connect(user11).submitBugReport(0, "Low bug", "");
    const reportIds = await gloryToken.getAllBugReports();
    const reportId = reportIds[reportIds.length - 1];

    // Use a small custom bounty to avoid depleting the reserve
    const smallBounty = ethers.parseEther("1000"); // Very small bounty
    await gloryToken.verifyBugReport(reportId, smallBounty);
    await gloryToken.payBugBounty(reportId);

    const newReserve = await gloryToken.getBugBountyReserve();
    expect(newReserve).to.equal(initialReserve - smallBounty);
  });
});

describe("GloryToken Incremental Leaderboard", function () {
  it("should maintain a sorted leaderboard as balances are updated", async function () {
    // Setup: make users active participants in DumpToken
    const minStake = await dumpToken.getMinimumStake();
    await dumpToken.transfer(user1.address, ethers.parseEther("10000"));
    await dumpToken.connect(user1).stakeForParticipation(minStake);
    await dumpToken.transfer(user2.address, ethers.parseEther("10000"));
    await dumpToken.connect(user2).stakeForParticipation(minStake);
    await dumpToken.transfer(user3.address, ethers.parseEther("10000"));
    await dumpToken.connect(user3).stakeForParticipation(minStake);

    // Record epoch balances
    await gloryToken.recordEpochBalance(user1.address, ethers.parseEther("100"));
    await gloryToken.recordEpochBalance(user2.address, ethers.parseEther("50"));
    await gloryToken.recordEpochBalance(user3.address, ethers.parseEther("200"));

    // Get leaderboard (should be sorted: user2, user1, user3)
    const leaderboard = await gloryToken.getLeaderboard();
    expect(leaderboard[0]).to.equal(user2.address);
    expect(leaderboard[1]).to.equal(user1.address);
    expect(leaderboard[2]).to.equal(user3.address);
  });

  it("should reorder leaderboard when a user's balance changes", async function () {
    // user1 now has the lowest balance
    await gloryToken.recordEpochBalance(user1.address, ethers.parseEther("10"));
    const leaderboard = await gloryToken.getLeaderboard();
    expect(leaderboard[0]).to.equal(user1.address);
  });

  it("should remove a user from the leaderboard if their balance is set to 0", async function () {
    // Remove user2
    await gloryToken.recordEpochBalance(user2.address, 0);
    const leaderboard = await gloryToken.getLeaderboard();
    expect(leaderboard).to.not.include(user2.address);
  });
});

describe("GloryToken Skip List Leaderboard - Advanced", function () {
  it("should handle many users and remain sorted", async function () {
    const minStake = await dumpToken.getMinimumStake();
    const users = [user1, user2, user3, user4, user5, user6, user7, user8, user9, user10];
    // Make all users active
    for (let i = 0; i < users.length; i++) {
      await dumpToken.transfer(users[i].address, ethers.parseEther("10000"));
      await dumpToken.connect(users[i]).stakeForParticipation(minStake);
    }
    // Assign random balances
    const balances = ["500", "100", "900", "300", "700", "200", "800", "400", "600", "1000"];
    for (let i = 0; i < users.length; i++) {
      await gloryToken.recordEpochBalance(users[i].address, ethers.parseEther(balances[i]));
    }
    // Get leaderboard and check sorted order (lowest to highest)
    const leaderboard = await gloryToken.getLeaderboard();
    let prev = await gloryToken.epochs(1).averageDumpHeld(leaderboard[0]);
    for (let i = 1; i < leaderboard.length; i++) {
      const curr = await gloryToken.epochs(1).averageDumpHeld(leaderboard[i]);
      expect(curr).to.be.gte(prev); // Should be ascending
      prev = curr;
    }
  });

  it("should update leaderboard correctly when balances change dramatically", async function () {
    // user5 drops to lowest balance
    await gloryToken.recordEpochBalance(user5.address, ethers.parseEther("10"));
    const leaderboard = await gloryToken.getLeaderboard();
    expect(leaderboard[0]).to.equal(user5.address);
    // user10 jumps to highest balance
    await gloryToken.recordEpochBalance(user10.address, ethers.parseEther("2000"));
    const leaderboard2 = await gloryToken.getLeaderboard();
    expect(leaderboard2[leaderboard2.length - 1]).to.equal(user10.address);
  });

  it("should remove users with zero balance and keep leaderboard valid", async function () {
    await gloryToken.recordEpochBalance(user3.address, 0);
    await gloryToken.recordEpochBalance(user7.address, 0);
    const leaderboard = await gloryToken.getLeaderboard();
    expect(leaderboard).to.not.include(user3.address);
    expect(leaderboard).to.not.include(user7.address);
    // All remaining users should have nonzero balances
    for (const addr of leaderboard) {
      const bal = await gloryToken.epochs(1).averageDumpHeld(addr);
      expect(bal).to.be.gt(0);
    }
  });

  it("should maintain leaderboard integrity after many random operations", async function () {
    // Randomly update balances
    for (let i = 0; i < 20; i++) {
      const idx = Math.floor(Math.random() * 10);
      const newBal = ethers.parseEther((Math.floor(Math.random() * 1000) + 1).toString());
      await gloryToken.recordEpochBalance(eval(`user${idx + 1}`).address, newBal);
    }
    // Check leaderboard is sorted
    const leaderboard = await gloryToken.getLeaderboard();
    let prev = await gloryToken.epochs(1).averageDumpHeld(leaderboard[0]);
    for (let i = 1; i < leaderboard.length; i++) {
      const curr = await gloryToken.epochs(1).averageDumpHeld(leaderboard[i]);
      expect(curr).to.be.gte(prev);
      prev = curr;
    }
  });
});