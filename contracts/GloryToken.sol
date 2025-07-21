// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "./DumpToken.sol";

/**
 * @title GloryToken
 * @dev Reward token for DUMP/GLORY game - awarded to those who hold least DUMP
 * @author Prime Anomaly
 */
contract GloryToken is ERC20, ReentrancyGuard {
    using SafeMath for uint256;

    // Constants
    uint256 public constant BUG_BOUNTY_PERCENTAGE = 5; // 5% for bug bounty
    uint256 public constant EPOCH_DURATION = 30 days;
    
    // Reward distribution tiers (basis points - 10000 = 100%)
    uint256 public constant WINNER_PERCENTAGE = 4000;      // 40% to winner
    uint256 public constant TOP_TIER_PERCENTAGE = 4000;    // 40% to top 2-10%
    uint256 public constant MIDDLE_TIER_PERCENTAGE = 2000; // 20% to middle 40%
    uint256 public constant BOTTOM_TIER_PERCENTAGE = 500;  // 5% to bottom 50%
    
    // State variables
    DumpToken public dumpToken;
    uint256 public currentEpoch;
    uint256 public epochStartTime;
    uint256 public bugBountyReserve;
    uint256 public totalRewardsDistributed;
    
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
    event BugBountyClaimed(address indexed finder, uint256 amount, string description);
    event EpochSnapshot(uint256 epoch, uint256 totalParticipants);
    
    // Modifiers
    modifier onlyDumpToken() {
        require(msg.sender == address(dumpToken), "Only DUMP token can call");
        _;
    }
    
    constructor(
        string memory name,
        string memory symbol,
        address _dumpToken
    ) ERC20(name, symbol) {
        dumpToken = DumpToken(_dumpToken);
        currentEpoch = 1;
        epochStartTime = block.timestamp;
        
        // Reserve 5% for bug bounty
        uint256 totalSupply = 1000000 * 10**decimals(); // 1M GLORY
        bugBountyReserve = totalSupply.mul(BUG_BOUNTY_PERCENTAGE).div(100);
        _mint(address(this), bugBountyReserve);
    }
    
    /**
     * @dev Record a user's average DUMP balance for the current epoch
     * @param user Address to record
     * @param averageBalance Average DUMP balance over the epoch
     */
    function recordEpochBalance(address user, uint256 averageBalance) external onlyDumpToken {
        EpochData storage epoch = epochs[currentEpoch];
        
        // Add user to participants if not already recorded
        if (epoch.averageDumpHeld[user] == 0) {
            epoch.participants.push(user);
            epoch.totalParticipants = epoch.totalParticipants.add(1);
        }
        
        epoch.averageDumpHeld[user] = averageBalance;
    }
    
    /**
     * @dev Finalize epoch and distribute GLORY rewards to ALL participants
     */
    function finalizeEpoch() external nonReentrant {
        require(block.timestamp >= epochStartTime.add(EPOCH_DURATION), "Epoch not over yet");
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
        uint256 topTierCount = totalParticipants.mul(10).div(100);     // Top 10%
        uint256 middleTierCount = totalParticipants.mul(40).div(100);  // Next 40%
        uint256 bottomTierCount = totalParticipants.sub(topTierCount).sub(middleTierCount); // Bottom 50%
        
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
            totalRewardsDistributed = totalRewardsDistributed.add(reward);
            
            // Mint GLORY to participant
            _mint(participant, reward);
        }
        
        // Increment epoch
        currentEpoch = currentEpoch.add(1);
        epochStartTime = block.timestamp;
        
        emit EpochRewardsDistributed(currentEpoch.sub(1), allWinners, allAmounts);
        emit EpochSnapshot(currentEpoch.sub(1), epoch.totalParticipants);
    }
    
    /**
     * @dev Calculate total rewards for the epoch
     * @return Total rewards available
     */
    function calculateEpochRewards() internal view returns (uint256) {
        // Base reward per epoch (can be adjusted)
        uint256 baseReward = 10000 * 10**decimals(); // 10k GLORY per epoch
        
        // Add any additional rewards from fee pot buyback
        // TODO: Implement fee pot buyback mechanism
        
        return baseReward;
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
        uint256 topTierCount = totalParticipants.mul(10).div(100);     // Top 10%
        uint256 middleTierCount = totalParticipants.mul(40).div(100);  // Next 40%
        
        // Determine which tier this rank belongs to
        if (rank == 0) {
            // 🏆 WINNER (Rank 0) - Gets 40% of total rewards
            return totalRewards.mul(WINNER_PERCENTAGE).div(10000);
            
        } else if (rank < topTierCount) {
            // 🥈 TOP TIER (Ranks 1-9%) - Share 40% of total rewards
            uint256 topTierRewards = totalRewards.mul(TOP_TIER_PERCENTAGE).div(10000);
            uint256 participantsInTier = topTierCount.sub(1); // Exclude winner
            if (participantsInTier == 0) participantsInTier = 1;
            
            // Exponential decay within top tier
            uint256 baseReward = topTierRewards.div(participantsInTier.mul(3)); // Conservative base
            uint256 multiplier = 200; // 2x decay per rank
            uint256 power = participantsInTier.sub(rank);
            
            uint256 reward = baseReward;
            for (uint256 i = 0; i < power; i++) {
                reward = reward.mul(multiplier).div(100);
            }
            
            return reward;
            
        } else if (rank < topTierCount.add(middleTierCount)) {
            // 🥉 MIDDLE TIER (Ranks 10-49%) - Share 20% of total rewards
            uint256 middleTierRewards = totalRewards.mul(MIDDLE_TIER_PERCENTAGE).div(10000);
            uint256 participantsInTier = middleTierCount;
            if (participantsInTier == 0) participantsInTier = 1;
            
            // Linear decay within middle tier
            uint256 baseReward = middleTierRewards.div(participantsInTier);
            uint256 rankInTier = rank.sub(topTierCount);
            uint256 decayFactor = participantsInTier.sub(rankInTier).mul(100).div(participantsInTier);
            
            return baseReward.mul(decayFactor).div(100);
            
        } else {
            // 📉 BOTTOM TIER (Ranks 50%+) - Share 5% of total rewards
            uint256 bottomTierRewards = totalRewards.mul(BOTTOM_TIER_PERCENTAGE).div(10000);
            uint256 participantsInTier = totalParticipants.sub(topTierCount).sub(middleTierCount);
            if (participantsInTier == 0) participantsInTier = 1;
            
            // Minimal rewards for bottom tier
            uint256 baseReward = bottomTierRewards.div(participantsInTier);
            uint256 rankInTier = rank.sub(topTierCount).sub(middleTierCount);
            uint256 decayFactor = participantsInTier.sub(rankInTier).mul(50).div(participantsInTier);
            
            return baseReward.mul(decayFactor).div(100);
        }
    }
    
    /**
     * @dev Sort participants by average DUMP held (ascending)
     * @param epochNumber Epoch to sort
     * @return Sorted array of participant addresses
     */
    function sortParticipantsByDumpHeld(uint256 epochNumber) internal view returns (address[] memory) {
        EpochData storage epoch = epochs[epochNumber];
        address[] memory participants = epoch.participants;
        
        // Simple bubble sort (for small datasets)
        for (uint256 i = 0; i < participants.length; i++) {
            for (uint256 j = i + 1; j < participants.length; j++) {
                if (epoch.averageDumpHeld[participants[i]] > epoch.averageDumpHeld[participants[j]]) {
                    address temp = participants[i];
                    participants[i] = participants[j];
                    participants[j] = temp;
                }
            }
        }
        
        return participants;
    }
    
    /**
     * @dev Claim bug bounty
     * @param description Description of the bug found
     */
    function claimBugBounty(string memory description) external nonReentrant {
        require(bugBountyReserve > 0, "No bug bounty available");
        require(bytes(description).length > 0, "Description required");
        
        uint256 bounty = bugBountyReserve;
        bugBountyReserve = 0;
        
        _transfer(address(this), msg.sender, bounty);
        
        emit BugBountyClaimed(msg.sender, bounty, description);
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
                rank = rank.add(1);
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
        uint256 epochEndTime = epochStartTime.add(EPOCH_DURATION);
        if (block.timestamp >= epochEndTime) return 0;
        return epochEndTime.sub(block.timestamp);
    }
    
    /**
     * @dev Get total participants in current epoch
     * @return Number of participants
     */
    function getCurrentEpochParticipants() external view returns (uint256) {
        return epochs[currentEpoch].totalParticipants;
    }
}