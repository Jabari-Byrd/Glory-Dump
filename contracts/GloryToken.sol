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
    uint256 public constant TOP_PERCENTAGE = 5; // Top 5% get rewards
    uint256 public constant BUG_BOUNTY_PERCENTAGE = 5; // 5% for bug bounty
    uint256 public constant EPOCH_DURATION = 30 days;
    
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
     * @dev Finalize epoch and distribute GLORY rewards
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
        
        // Calculate top 5% threshold
        uint256 topCount = epoch.totalParticipants.mul(TOP_PERCENTAGE).div(100);
        if (topCount == 0) topCount = 1; // At least one winner
        
        // Distribute rewards
        address[] memory winners = new address[](topCount);
        uint256[] memory amounts = new uint256[](topCount);
        
        for (uint256 i = 0; i < topCount; i++) {
            address winner = sortedParticipants[i];
            uint256 reward = calculateReward(i, topCount, epochRewards);
            
            winners[i] = winner;
            amounts[i] = reward;
            
            epoch.gloryRewarded[winner] = reward;
            totalRewardsDistributed = totalRewardsDistributed.add(reward);
            
            // Mint GLORY to winner
            _mint(winner, reward);
        }
        
        // Increment epoch
        currentEpoch = currentEpoch.add(1);
        epochStartTime = block.timestamp;
        
        emit EpochRewardsDistributed(currentEpoch.sub(1), winners, amounts);
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
     * @dev Calculate individual reward based on rank
     * @param rank Position in leaderboard (0 = lowest DUMP holder)
     * @param totalWinners Total number of winners
     * @param totalRewards Total rewards to distribute
     * @return Individual reward amount
     */
    function calculateReward(uint256 rank, uint256 totalWinners, uint256 totalRewards) internal pure returns (uint256) {
        if (totalWinners == 0) return 0;
        
        // Logarithmic distribution: higher ranks get exponentially more
        // Formula: reward = base * (1.5 ^ (totalWinners - rank - 1))
        uint256 baseReward = totalRewards.div(totalWinners.mul(2)); // Conservative base
        
        if (rank == 0) {
            // Winner gets 50% of total rewards
            return totalRewards.div(2);
        } else {
            // Others get decreasing amounts
            uint256 multiplier = 150; // 1.5x in basis points
            uint256 power = totalWinners.sub(rank).sub(1);
            
            uint256 reward = baseReward;
            for (uint256 i = 0; i < power; i++) {
                reward = reward.mul(multiplier).div(100);
            }
            
            return reward;
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