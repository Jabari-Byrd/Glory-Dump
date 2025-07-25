// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./DumpToken.sol";

/**
 * @title GloryToken
 * @dev Reward token for DUMP/GLORY game - awarded to those who hold least DUMP
 * @author Prime Anomaly
 */
contract GloryToken is ERC20, ReentrancyGuard, Ownable {

    // Constants
    uint256 public constant BUG_BOUNTY_PERCENTAGE = 10; // 10% for bug bounty
    uint256 public constant EPOCH_DURATION = 30 days;
    
    // Reward distribution tiers (basis points - 10000 = 100%)
    uint256 public constant WINNER_PERCENTAGE = 4000;      // 40% to winner
    uint256 public constant TOP_TIER_PERCENTAGE = 4000;    // 40% to top 2-10%
    uint256 public constant MIDDLE_TIER_PERCENTAGE = 2000; // 20% to middle 40%
    uint256 public constant BOTTOM_TIER_PERCENTAGE = 500;  // 5% to bottom 50%
    
    // Bug bounty severity levels and payouts
    enum BugSeverity { LOW, MEDIUM, HIGH, CRITICAL }
    
    uint256 public constant CRITICAL_BOUNTY = 100000 * 10**18; // 100K GLORY
    uint256 public constant HIGH_BOUNTY = 50000 * 10**18;      // 50K GLORY
    uint256 public constant MEDIUM_BOUNTY = 25000 * 10**18;    // 25K GLORY
    uint256 public constant LOW_BOUNTY = 10000 * 10**18;       // 10K GLORY
    
    // State variables
    DumpToken public dumpToken;
    uint256 public currentEpoch;
    uint256 public epochStartTime;
    uint256 public bugBountyReserve;
    uint256 public totalRewardsDistributed;
    
    // Add for bonus epoch logic
    uint256 public highestEpochVolume;
    uint256 public highestUniquePlayers;
    mapping(uint256 => bool) public isBonusEpoch;
    event BonusEpoch(uint256 indexed epoch, uint256 gloryAmount, string reason);
    event BurnedEpoch(uint256 indexed epoch, uint256 gloryAmount);
    
    // Bonus triggers and multipliers
    uint256 public milestoneThresholdPercent = 10; // 10% increase required
    uint256 public highestTransfers;
    uint256 public highestActiveAddresses;
    uint256 public highestLiquidity;
    uint256 public highestGloryPrice;
    uint256 public highestDumpVolatility;
    event UltraBonusEpoch(uint256 indexed epoch, uint256 gloryAmount, uint256 multiplier, string reason);
    
    // Bug bounty system
    struct BugReport {
        address reporter;
        BugSeverity severity;
        string description;
        string proofOfConcept;
        uint256 timestamp;
        bool verified;
        bool paid;
        uint256 bountyAmount;
    }
    
    mapping(bytes32 => BugReport) public bugReports;
    mapping(address => uint256) public reporterTotalBounties;
    bytes32[] public allBugReports;
    
    // Epoch data
    struct EpochData {
        uint256 totalParticipants;
        uint256 totalRewards;
        mapping(address => uint256) averageDumpHeld;
        mapping(address => uint256) gloryRewarded;
        address[] participants;
        bool finalized;
    }
    
    mapping(uint256 => EpochData) public epochs;
    mapping(uint256 => bool) public isEpochFinalized;
    
    // Events
    event EpochRewardsDistributed(uint256 epoch, address[] winners, uint256[] amounts);
    event BugReportSubmitted(bytes32 indexed reportId, address indexed reporter, BugSeverity severity, string description);
    event BugReportVerified(bytes32 indexed reportId, address indexed reporter, uint256 bountyAmount);
    event BugBountyPaid(bytes32 indexed reportId, address indexed reporter, uint256 amount);
    event EpochSnapshot(uint256 epoch, uint256 totalParticipants);
    
    // Modifiers
    modifier onlyDumpToken() {
        require(msg.sender == address(dumpToken), "Only DUMP token can call");
        _;
    }
    
    modifier onlyOwnerOrDumpToken() {
        require(msg.sender == owner() || msg.sender == address(dumpToken), "Only owner or DUMP token can call");
        _;
    }
    
    constructor(
        string memory name,
        string memory symbol,
        address _dumpToken
    ) ERC20(name, symbol) Ownable(msg.sender) {
        dumpToken = DumpToken(_dumpToken);
        currentEpoch = 1;
        epochStartTime = block.timestamp;
        
        // Mint full supply to owner, then reserve 5% for bug bounty
        uint256 totalSupply = 1000000 * 10**decimals(); // 1M GLORY
        _mint(msg.sender, totalSupply);
        
        bugBountyReserve = totalSupply * BUG_BOUNTY_PERCENTAGE / 100;
        _transfer(msg.sender, address(this), bugBountyReserve);
    }
    
    // --- Incremental Leaderboard (Skip List) ---
    uint256 private constant MAX_LEVEL = 8; // Supports up to 2^8 = 256 levels (enough for thousands of users)
    struct SkipNode {
        address user;
        uint256 balance;
        address[MAX_LEVEL] forward;
        bool exists;
        uint8 level;
    }
    mapping(address => SkipNode) public skipNodes;
    address public skipListHead;
    uint256 public skipListSize;

    // Pseudo-random level generator (deterministic, not secure, but fine for leaderboard)
    function randomLevel(address user, uint256 balance) internal pure returns (uint8) {
        uint256 hash = uint256(keccak256(abi.encodePacked(user, balance)));
        uint8 lvl = 1;
        while ((hash & 0xFF) < 64 && lvl < MAX_LEVEL) { // 25% chance to go up each level
            lvl++;
            hash >>= 8;
        }
        return lvl;
    }

    function updateSkipList(address user, uint256 newBalance) internal {
        if (skipNodes[user].exists) {
            removeFromSkipList(user);
        }
        insertIntoSkipList(user, newBalance);
    }

    function insertIntoSkipList(address user, uint256 balance) internal {
        uint8 lvl = randomLevel(user, balance);
        address[MAX_LEVEL] memory update;
        address x = skipListHead;
        for (uint8 i = MAX_LEVEL; i > 0; i--) {
            while (skipNodes[skipNodes[x].forward[i-1]].exists && skipNodes[skipNodes[x].forward[i-1]].balance < balance) {
                x = skipNodes[x].forward[i-1];
            }
            update[i-1] = x;
        }
        SkipNode storage node = skipNodes[user];
        node.user = user;
        node.balance = balance;
        node.exists = true;
        node.level = lvl;
        for (uint8 i = 0; i < lvl; i++) {
            node.forward[i] = skipNodes[update[i]].forward[i];
            skipNodes[update[i]].forward[i] = user;
        }
        skipListSize++;
    }

    function removeFromSkipList(address user) internal {
        if (!skipNodes[user].exists) return;
        address[MAX_LEVEL] memory update;
        address x = skipListHead;
        uint256 balance = skipNodes[user].balance;
        for (uint8 i = MAX_LEVEL; i > 0; i--) {
            while (skipNodes[skipNodes[x].forward[i-1]].exists && skipNodes[skipNodes[x].forward[i-1]].balance < balance) {
                x = skipNodes[x].forward[i-1];
            }
            update[i-1] = x;
        }
        for (uint8 i = 0; i < skipNodes[user].level; i++) {
            if (skipNodes[update[i]].forward[i] == user) {
                skipNodes[update[i]].forward[i] = skipNodes[user].forward[i];
            }
        }
        skipNodes[user].exists = false;
        skipListSize--;
    }

    // Override recordEpochBalance to update skip list leaderboard
    function recordEpochBalance(address user, uint256 averageBalance) external onlyDumpToken {
        EpochData storage epoch = epochs[currentEpoch];
        if (epoch.averageDumpHeld[user] == 0) {
            epoch.participants.push(user);
            epoch.totalParticipants = epoch.totalParticipants + 1;
        }
        epoch.averageDumpHeld[user] = averageBalance;
        updateSkipList(user, averageBalance);
    }
    
    /**
     * @dev Finalize epoch and distribute GLORY rewards to ALL participants
     */
    function finalizeEpoch() external nonReentrant {
        require(block.timestamp >= epochStartTime + EPOCH_DURATION, "Epoch not over yet");
        require(!isEpochFinalized[currentEpoch], "Epoch already finalized");
        
        EpochData storage epoch = epochs[currentEpoch];
        require(!epoch.finalized, "Epoch already finalized");
        
        // Mark as finalized
        epoch.finalized = true;
        isEpochFinalized[currentEpoch] = true;
        
        // Calculate rewards
        uint256 epochRewards = calculateEpochRewards();
        epoch.totalRewards = epochRewards;
        
        // Sort participants by average DUMP held (ascending - lowest first)
        address[] memory sortedParticipants = sortParticipantsByDumpHeld(currentEpoch);
        
        // Calculate tier thresholds
        uint256 totalParticipants = epoch.totalParticipants;
        uint256 topTierCount = totalParticipants * 10 / 100;     // Top 10%
        uint256 middleTierCount = totalParticipants * 40 / 100;  // Next 40%
        uint256 bottomTierCount = totalParticipants - topTierCount - middleTierCount; // Bottom 50%
        
        // Ensure minimum counts
        if (topTierCount == 0) topTierCount = 1;
        if (middleTierCount == 0) middleTierCount = 1;
        if (bottomTierCount == 0) bottomTierCount = 1;
        
        // Distribute rewards to ALL participants
        address[] memory allWinners = new address[](totalParticipants);
        uint256[] memory allAmounts = new uint256[](totalParticipants);
        
        for (uint256 i = 0; i < totalParticipants; i++) {
            address participant = sortedParticipants[i];
            uint256 reward = calculateReward(i, totalParticipants, epochRewards);
            
            allWinners[i] = participant;
            allAmounts[i] = reward;
            
            epoch.gloryRewarded[participant] = reward;
            totalRewardsDistributed = totalRewardsDistributed + reward;
            
            // Mint GLORY to participant
            _mint(participant, reward);
        }
        
        // Increment epoch
        currentEpoch = currentEpoch + 1;
        epochStartTime = block.timestamp;
        
        emit EpochRewardsDistributed(currentEpoch - 1, allWinners, allAmounts);
        emit EpochSnapshot(currentEpoch - 1, epoch.totalParticipants);
    }
    
    /**
     * @dev Calculate total rewards for the epoch (dynamic: deflationary by default, rare bonus epochs)
     * @return Total rewards available
     */
    function calculateEpochRewards() internal returns (uint256) {
        uint256 baseReward = 10000 * 10**decimals();
        uint256 buybackReward = getBuybackGlory();
        (uint256 bonusMultiplier, string memory reason, bool ultra) = checkBonusEpoch();
        uint256 totalReward = baseReward;
        if (bonusMultiplier > 0) {
            totalReward += buybackReward * bonusMultiplier;
            if (ultra) {
                emit UltraBonusEpoch(currentEpoch, buybackReward * bonusMultiplier, bonusMultiplier, reason);
            } else {
                emit BonusEpoch(currentEpoch, buybackReward * bonusMultiplier, reason);
            }
        } else {
            if (buybackReward > 0) {
                _burn(address(this), buybackReward);
                emit BurnedEpoch(currentEpoch, buybackReward);
            }
        }
        return totalReward;
    }

    // Helper to get GLORY from FeePot (assumes FeePot transfers GLORY to this contract after buyback)
    function getBuybackGlory() internal view returns (uint256) {
        // For simplicity, assume all GLORY sent to this contract (not bug bounty reserve) is buyback
        uint256 contractBalance = balanceOf(address(this));
        return contractBalance > bugBountyReserve ? contractBalance - bugBountyReserve : 0;
    }

    // --- Bonus Epoch Logic ---
    function checkBonusEpoch() internal returns (uint256, string memory, bool) {
        uint256 triggers = 0;
        string memory reasons = "";
        // 1. Volume milestone
        uint256 epochVolume = dumpToken.getEpochVolume(currentEpoch);
        if (epochVolume > highestEpochVolume + (highestEpochVolume * milestoneThresholdPercent / 100)) {
            highestEpochVolume = epochVolume;
            triggers++;
            reasons = string(abi.encodePacked(reasons, "Volume milestone! "));
        }
        // 2. Unique players milestone
        uint256 uniquePlayers = dumpToken.getUniquePlayers(currentEpoch);
        if (uniquePlayers > highestUniquePlayers + (highestUniquePlayers * milestoneThresholdPercent / 100)) {
            highestUniquePlayers = uniquePlayers;
            triggers++;
            reasons = string(abi.encodePacked(reasons, "Unique players milestone! "));
        }
        // 3. Most active addresses
        uint256 activeAddresses = dumpToken.getActiveAddresses(currentEpoch);
        if (activeAddresses > highestActiveAddresses + (highestActiveAddresses * milestoneThresholdPercent / 100)) {
            highestActiveAddresses = activeAddresses;
            triggers++;
            reasons = string(abi.encodePacked(reasons, "Active address milestone! "));
        }
        // 4. GLORY price increase
        uint256 gloryPrice = feePot.getGloryPrice();
        if (gloryPrice > highestGloryPrice + (highestGloryPrice * milestoneThresholdPercent / 100)) {
            highestGloryPrice = gloryPrice;
            triggers++;
            reasons = string(abi.encodePacked(reasons, "GLORY price milestone! "));
        }
        // 5. DUMP price volatility
        uint256 dumpVolatility = feePot.getDumpVolatility();
        if (dumpVolatility > highestDumpVolatility + (highestDumpVolatility * milestoneThresholdPercent / 100)) {
            highestDumpVolatility = dumpVolatility;
            triggers++;
            reasons = string(abi.encodePacked(reasons, "DUMP volatility milestone! "));
        }
        // 6. Liquidity in the pool
        uint256 liquidity = feePot.getLiquidity();
        if (liquidity > highestLiquidity + (highestLiquidity * milestoneThresholdPercent / 100)) {
            highestLiquidity = liquidity;
            triggers++;
            reasons = string(abi.encodePacked(reasons, "Liquidity milestone! "));
        }
        // 7. Random bonus (rarer: 1 in 50)
        bool random = (uint256(keccak256(abi.encodePacked(blockhash(block.number - 1), currentEpoch, block.timestamp))) % 50 == 0);
        if (random) {
            triggers++;
            reasons = string(abi.encodePacked(reasons, "Random bonus! "));
        }
        // 8. Easter egg: blockhash last 3 digits same
        uint256 hashNum = uint256(blockhash(block.number - 1));
        if (hasRepeatingDigits(hashNum, 3)) {
            triggers++;
            reasons = string(abi.encodePacked(reasons, "Repeating digits! "));
        }
        // 9. Easter egg: epoch palindrome
        if (isPalindrome(currentEpoch)) {
            triggers++;
            reasons = string(abi.encodePacked(reasons, "Palindrome epoch! "));
        }
        // Multiplier: 1x for 1, 2x for 2, 3x for 3+ (ultra bonus)
        uint256 bonusMultiplier = triggers;
        bool ultra = triggers >= 3;
        isBonusEpoch[currentEpoch] = (bonusMultiplier > 0);
        return (bonusMultiplier, reasons, ultra);
    }

    // --- Helper functions ---
    function hasRepeatingDigits(uint256 number, uint8 count) internal pure returns (bool) {
        uint256 lastDigit = number % 10;
        for (uint8 i = 1; i < count; i++) {
            number /= 10;
            if (number % 10 != lastDigit) return false;
        }
        return true;
    }
    function isPalindrome(uint256 num) internal pure returns (bool) {
        uint256 reversed = 0;
        uint256 original = num;
        while (num != 0) {
            reversed = reversed * 10 + num % 10;
            num /= 10;
        }
        return original == reversed;
    }
    
    /**
     * @dev Calculate individual reward based on rank with real-world wealth distribution
     * @param rank Position in leaderboard (0 = lowest DUMP holder = winner)
     * @param totalParticipants Total number of participants
     * @param totalRewards Total rewards to distribute
     * @return Individual reward amount
     */
    function calculateReward(uint256 rank, uint256 totalParticipants, uint256 totalRewards) internal pure returns (uint256) {
        if (totalParticipants == 0) return 0;
        
        // Calculate tier thresholds
        uint256 topTierCount = totalParticipants * 10 / 100;     // Top 10%
        uint256 middleTierCount = totalParticipants * 40 / 100;  // Next 40%
        
        // Determine which tier this rank belongs to
        if (rank == 0) {
            // 🏆 WINNER (Rank 0) - Gets 40% of total rewards
            return totalRewards * WINNER_PERCENTAGE / 10000;
            
        } else if (rank < topTierCount) {
            // 🥈 TOP TIER (Ranks 1-9%) - Share 40% of total rewards
            uint256 topTierRewards = totalRewards * TOP_TIER_PERCENTAGE / 10000;
            uint256 participantsInTier = topTierCount - 1; // Exclude winner
            if (participantsInTier == 0) participantsInTier = 1;
            
            // Exponential decay within top tier
            uint256 baseReward = topTierRewards / (participantsInTier * 3); // Conservative base
            uint256 multiplier = 200; // 2x decay per rank
            uint256 power = participantsInTier - rank;
            
            uint256 reward = baseReward;
            for (uint256 i = 0; i < power; i++) {
                reward = reward * multiplier / 100;
            }
            
            return reward;
            
        } else if (rank < topTierCount + middleTierCount) {
            // 🥉 MIDDLE TIER (Ranks 10-49%) - Share 20% of total rewards
            uint256 middleTierRewards = totalRewards * MIDDLE_TIER_PERCENTAGE / 10000;
            uint256 participantsInTier = middleTierCount;
            if (participantsInTier == 0) participantsInTier = 1;
            
            // Linear decay within middle tier
            uint256 baseReward = middleTierRewards / participantsInTier;
            uint256 rankInTier = rank - topTierCount;
            uint256 decayFactor = (participantsInTier - rankInTier) * 100 / participantsInTier;
            
            return baseReward * decayFactor / 100;
            
        } else {
            // 📉 BOTTOM TIER (Ranks 50%+) - Share 5% of total rewards
            uint256 bottomTierRewards = totalRewards * BOTTOM_TIER_PERCENTAGE / 10000;
            uint256 participantsInTier = totalParticipants - topTierCount - middleTierCount;
            if (participantsInTier == 0) participantsInTier = 1;
            
            // Minimal rewards for bottom tier
            uint256 baseReward = bottomTierRewards / participantsInTier;
            uint256 rankInTier = rank - topTierCount - middleTierCount;
            uint256 decayFactor = (participantsInTier - rankInTier) * 50 / participantsInTier;
            
            return baseReward * decayFactor / 100;
        }
    }
    
    // Return sorted participants from skip list
    function sortParticipantsByDumpHeld(uint256 /*epochNumber*/) internal view returns (address[] memory) {
        address[] memory participants = new address[](skipListSize);
        address current = skipNodes[skipListHead].forward[0];
        uint256 idx = 0;
        while (current != address(0)) {
            participants[idx] = current;
            current = skipNodes[current].forward[0];
            idx++;
        }
        return participants;
    }
    
    /**
     * @dev Submit a bug report
     * @param severity Severity of the bug
     * @param description Description of the bug
     * @param proofOfConcept Proof of concept (optional)
     */
    function submitBugReport(BugSeverity severity, string memory description, string memory proofOfConcept) external nonReentrant {
        bytes32 reportId = keccak256(abi.encodePacked(msg.sender, block.timestamp, description));
        require(bugReports[reportId].reporter == address(0), "Bug report already submitted");
        
        BugReport memory newReport = BugReport({
            reporter: msg.sender,
            severity: severity,
            description: description,
            proofOfConcept: proofOfConcept,
            timestamp: block.timestamp,
            verified: false,
            paid: false,
            bountyAmount: 0
        });
        
        bugReports[reportId] = newReport;
        allBugReports.push(reportId);
        reporterTotalBounties[msg.sender] = reporterTotalBounties[msg.sender] + 1;
        
        emit BugReportSubmitted(reportId, msg.sender, severity, description);
    }
    
    /**
     * @dev Verify a bug report and set bounty amount (owner only)
     * @param reportId ID of the bug report to verify
     * @param customBountyAmount Custom bounty amount (0 for standard amounts)
     */
    function verifyBugReport(bytes32 reportId, uint256 customBountyAmount) external onlyOwner {
        BugReport storage report = bugReports[reportId];
        require(report.reporter != address(0), "Bug report does not exist");
        require(!report.verified, "Bug report already verified");
        require(!report.paid, "Bug bounty already paid");
        
        // Mark as verified
        report.verified = true;
        
        // Calculate bounty amount
        uint256 bountyAmount = customBountyAmount;
        if (bountyAmount == 0) {
            if (report.severity == BugSeverity.CRITICAL) {
                bountyAmount = CRITICAL_BOUNTY;
            } else if (report.severity == BugSeverity.HIGH) {
                bountyAmount = HIGH_BOUNTY;
            } else if (report.severity == BugSeverity.MEDIUM) {
                bountyAmount = MEDIUM_BOUNTY;
            } else if (report.severity == BugSeverity.LOW) {
                bountyAmount = LOW_BOUNTY;
            }
        }
        
        require(bountyAmount <= bugBountyReserve, "Insufficient bug bounty reserve");
        require(bountyAmount > 0, "Bounty amount must be greater than 0");
        
        report.bountyAmount = bountyAmount;
        
        emit BugReportVerified(reportId, report.reporter, bountyAmount);
    }
    
    /**
     * @dev Pay the bounty for a verified bug report (owner only)
     * @param reportId ID of the bug report to pay
     */
    function payBugBounty(bytes32 reportId) external onlyOwner {
        BugReport storage report = bugReports[reportId];
        require(report.verified, "Bug report not verified");
        require(!report.paid, "Bug bounty already paid");
        require(report.bountyAmount > 0, "No bounty amount set");
        require(report.bountyAmount <= bugBountyReserve, "Insufficient bug bounty reserve");
        
        // Mark as paid
        report.paid = true;
        bugBountyReserve = bugBountyReserve - report.bountyAmount;
        
        // Transfer bounty to the reporter
        _transfer(address(this), report.reporter, report.bountyAmount);
        
        emit BugBountyPaid(reportId, report.reporter, report.bountyAmount);
    }
    
    /**
     * @dev Reject a bug report (owner only)
     * @param reportId ID of the bug report to reject
     * @param reason Reason for rejection
     */
    function rejectBugReport(bytes32 reportId, string memory reason) external onlyOwner {
        BugReport storage report = bugReports[reportId];
        require(report.reporter != address(0), "Bug report does not exist");
        require(!report.verified, "Bug report already verified");
        require(!report.paid, "Bug bounty already paid");
        
        // Mark as rejected by setting bounty to 0
        report.bountyAmount = 0;
        report.verified = true;
        report.paid = true;
        
        emit BugReportVerified(reportId, report.reporter, 0);
    }
    
    /**
     * @dev Get bug report details
     * @param reportId ID of the bug report
     * @return reporter Address of the reporter
     * @return severity Severity level
     * @return description Bug description
     * @return proofOfConcept Proof of concept
     * @return timestamp When reported
     * @return verified If verified
     * @return paid If paid
     * @return bountyAmount Bounty amount
     */
    function getBugReport(bytes32 reportId) external view returns (
        address reporter,
        BugSeverity severity,
        string memory description,
        string memory proofOfConcept,
        uint256 timestamp,
        bool verified,
        bool paid,
        uint256 bountyAmount
    ) {
        BugReport storage report = bugReports[reportId];
        return (
            report.reporter,
            report.severity,
            report.description,
            report.proofOfConcept,
            report.timestamp,
            report.verified,
            report.paid,
            report.bountyAmount
        );
    }
    
    /**
     * @dev Get all bug reports
     * @return Array of bug report IDs
     */
    function getAllBugReports() external view returns (bytes32[] memory) {
        return allBugReports;
    }
    
    /**
     * @dev Get total bounties earned by a reporter
     * @param reporter Address of the reporter
     * @return Total bounties earned
     */
    function getReporterTotalBounties(address reporter) external view returns (uint256) {
        return reporterTotalBounties[reporter];
    }
    
    /**
     * @dev Get remaining bug bounty reserve
     * @return Remaining bounty reserve
     */
    function getBugBountyReserve() external view returns (uint256) {
        return bugBountyReserve;
    }
    
    /**
     * @dev Get user's rank in current epoch
     * @param user Address to check
     * @return Rank (0 = lowest DUMP holder, -1 if not participating)
     */
    function getUserRank(address user) external view returns (int256) {
        EpochData storage epoch = epochs[currentEpoch];
        
        if (epoch.averageDumpHeld[user] == 0) return -1;
        
        uint256 userBalance = epoch.averageDumpHeld[user];
        int256 rank = 0;
        
        for (uint256 i = 0; i < epoch.participants.length; i++) {
            address participant = epoch.participants[i];
            if (epoch.averageDumpHeld[participant] < userBalance) {
                rank = rank + 1;
            }
        }
        
        return rank;
    }
    
    /**
     * @dev Get current epoch leaderboard
     * @return Array of addresses sorted by DUMP held (ascending)
     */
    function getLeaderboard() external view returns (address[] memory) {
        return sortParticipantsByDumpHeld(currentEpoch);
    }
    
    /**
     * @dev Get user's average DUMP balance for current epoch
     * @param user Address to check
     * @return Average DUMP balance
     */
    function getUserAverageDumpHeld(address user) external view returns (uint256) {
        return epochs[currentEpoch].averageDumpHeld[user];
    }
    
    /**
     * @dev Get epoch time remaining
     * @return Time remaining in current epoch
     */
    function getEpochTimeRemaining() external view returns (uint256) {
        uint256 epochEndTime = epochStartTime + EPOCH_DURATION;
        if (block.timestamp >= epochEndTime) return 0;
        return epochEndTime - block.timestamp;
    }
    
    /**
     * @dev Get total participants in current epoch
     * @return Number of participants
     */
    function getCurrentEpochParticipants() external view returns (uint256) {
        return epochs[currentEpoch].totalParticipants;
    }
}