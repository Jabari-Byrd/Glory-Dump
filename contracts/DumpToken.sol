// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";

/**
 * @title DumpToken
 * @dev The reverse wealth token - the less you hold, the better you rank
 * @author Prime Anomaly
 */
contract DumpToken is ERC20, ReentrancyGuard {
    using SafeMath for uint256;

    // Constants
    uint256 public constant DAILY_DEMURRAGE_RATE = 100; // 1% = 100 basis points
    uint256 public constant DEMURRAGE_BASIS_POINTS = 10000;
    uint256 public constant TRANSFER_FEE_BASIS_POINTS = 30; // 0.3%
    uint256 public constant THEFT_FEE_BASIS_POINTS = 30; // 0.3% (same as transfer)
    uint256 public constant SYBIL_STAKE_PERCENTAGE = 5; // 0.05% of total supply
    uint256 public constant EPOCH_DURATION = 30 days;
    uint256 public constant GRIEF_TAX_EXPONENT = 3;
    uint256 public constant THEFT_COOLDOWN_MIN = 30; // 30 seconds minimum
    uint256 public constant THEFT_COOLDOWN_MAX = 1 hours; // 1 hour maximum
    uint256 public constant MAX_VIRTUAL_SIZE = 1000000; // Prevent overflow
    
    // State variables
    uint256 public totalSupply;
    uint256 public currentEpoch;
    uint256 public epochStartTime;
    uint256 public feePot;
    
    // Mappings
    mapping(address => uint256) public lastDemurrageUpdate;
    mapping(address => uint256) public stakedAmount;
    mapping(address => uint256) public lastTransferTime;
    mapping(address => uint256) public cooldownEndTime;
    mapping(address => uint256) public lastTheftTime;
    mapping(address => uint256) public theftCooldownEndTime;
    mapping(uint256 => bool) public isEpochFinalized;
    mapping(address => bool) public isActiveParticipant;
    
    // Events
    event DemurrageApplied(address indexed user, uint256 amount, uint256 newBalance);
    event StakeDeposited(address indexed user, uint256 amount);
    event StakeWithdrawn(address indexed user, uint256 amount);
    event TransferWithCooldown(address indexed from, address indexed to, uint256 amount, uint256 cooldown);
    event TheftExecuted(address indexed thief, address indexed victim, uint256 amount, uint256 fee, uint256 cooldown);
    event FeeCollected(uint256 amount, uint256 newFeePot);
    event EpochFinalized(uint256 epoch, address finalizer);
    
    // Modifiers
    modifier onlyActiveParticipant() {
        require(isActiveParticipant[msg.sender], "Must be active participant");
        _;
    }
    
    modifier onlyBridge() {
        // TODO: Add bridge address check
        _;
    }
    
    constructor(
        string memory name,
        string memory symbol,
        uint256 initialSupply
    ) ERC20(name, symbol) {
        totalSupply = initialSupply;
        currentEpoch = 1;
        epochStartTime = block.timestamp;
        _mint(msg.sender, initialSupply);
    }
    
    /**
     * @dev Apply demurrage to an address
     * @param user Address to apply demurrage to
     */
    function applyDemurrage(address user) public {
        uint256 lastUpdate = lastDemurrageUpdate[user];
        uint256 currentTime = block.timestamp;
        
        if (lastUpdate == 0) {
            lastDemurrageUpdate[user] = currentTime;
            return;
        }
        
        uint256 daysSinceUpdate = (currentTime - lastUpdate) / 1 days;
        if (daysSinceUpdate == 0) return;
        
        uint256 balance = balanceOf(user);
        if (balance == 0) {
            lastDemurrageUpdate[user] = currentTime;
            return;
        }
        
        // Calculate demurrage: balance * (1 - daily_rate)^days
        uint256 demurrageMultiplier = DEMURRAGE_BASIS_POINTS;
        for (uint256 i = 0; i < daysSinceUpdate; i++) {
            demurrageMultiplier = demurrageMultiplier.mul(DEMURRAGE_BASIS_POINTS.sub(DAILY_DEMURRAGE_RATE)).div(DEMURRAGE_BASIS_POINTS);
        }
        
        uint256 newBalance = balance.mul(demurrageMultiplier).div(DEMURRAGE_BASIS_POINTS);
        uint256 demurrageAmount = balance.sub(newBalance);
        
        if (demurrageAmount > 0) {
            _burn(user, demurrageAmount);
            emit DemurrageApplied(user, demurrageAmount, newBalance);
        }
        
        lastDemurrageUpdate[user] = currentTime;
    }
    
    /**
     * @dev Stake DUMP to become an active participant
     * @param amount Amount to stake
     */
    function stakeForParticipation(uint256 amount) external nonReentrant {
        require(amount >= getMinimumStake(), "Insufficient stake amount");
        require(!isActiveParticipant[msg.sender], "Already active participant");
        
        _transfer(msg.sender, address(this), amount);
        stakedAmount[msg.sender] = amount;
        isActiveParticipant[msg.sender] = true;
        lastDemurrageUpdate[msg.sender] = block.timestamp;
        
        emit StakeDeposited(msg.sender, amount);
    }
    
    /**
     * @dev Withdraw stake and become inactive
     */
    function withdrawStake() external nonReentrant onlyActiveParticipant {
        require(block.timestamp >= cooldownEndTime[msg.sender], "Cooldown not expired");
        
        // Apply demurrage to staked amount
        applyDemurrage(msg.sender);
        
        uint256 stakedBalance = stakedAmount[msg.sender];
        require(stakedBalance > 0, "No stake to withdraw");
        
        stakedAmount[msg.sender] = 0;
        isActiveParticipant[msg.sender] = false;
        
        _transfer(address(this), msg.sender, stakedBalance);
        
        emit StakeWithdrawn(msg.sender, stakedBalance);
    }
    
    /**
     * @dev Transfer with cooldown and fee mechanics
     */
    function transfer(address to, uint256 amount) public override returns (bool) {
        require(to != address(0), "Cannot transfer to zero address");
        require(amount > 0, "Cannot transfer zero amount");
        
        // Apply demurrage to sender
        applyDemurrage(msg.sender);
        
        // Check cooldown
        require(block.timestamp >= cooldownEndTime[msg.sender], "Transfer cooldown active");
        
        // Calculate transfer fee
        uint256 feeAmount = amount.mul(TRANSFER_FEE_BASIS_POINTS).div(DEMURRAGE_BASIS_POINTS);
        uint256 transferAmount = amount.sub(feeAmount);
        
        // Calculate cooldown based on amount and epoch timing
        uint256 cooldown = computeCooldown(amount);
        
        // Update cooldown
        cooldownEndTime[msg.sender] = block.timestamp.add(cooldown);
        
        // Transfer tokens
        _transfer(msg.sender, to, transferAmount);
        
        // Collect fee
        if (feeAmount > 0) {
            _transfer(msg.sender, address(this), feeAmount);
            feePot = feePot.add(feeAmount);
            emit FeeCollected(feeAmount, feePot);
        }
        
        emit TransferWithCooldown(msg.sender, to, amount, cooldown);
        return true;
    }
    
    /**
     * @dev Compute cooldown based on transfer amount and epoch timing
     * @param amount Amount being transferred
     * @return cooldown Cooldown duration in seconds
     */
    function computeCooldown(uint256 amount) public view returns (uint256) {
        uint256 tMin = 10; // 10 seconds minimum
        uint256 tMax = EPOCH_DURATION;
        uint256 k = 2; // Quadratic scaling
        
        if (amount == 0) return tMin;
        
        uint256 scaled = amount.mul(1e18).div(totalSupply);
        uint256 baseCooldown = tMin.add(
            tMax.sub(tMin).mul(scaled.pow(k)).div(1e18.pow(k))
        );
        
        // Apply epoch-weighted scaling to cooldowns
        uint256 timeUntilEpochEnd = getEpochTimeRemaining();
        uint256 epochDuration = EPOCH_DURATION;
        
        if (timeUntilEpochEnd < 1 hours) {
            // Last hour: cooldowns become much longer (anti-spam)
            uint256 timeRatio = timeUntilEpochEnd.mul(1e18).div(1 hours);
            uint256 multiplier = 1e18.add(4e18.mul(1e18.sub(timeRatio)).div(1e18)); // 1x to 5x cooldown
            baseCooldown = baseCooldown.mul(multiplier).div(1e18);
            
        } else if (timeUntilEpochEnd < 1 days) {
            // Last day: moderate cooldown increase
            uint256 timeRatio = timeUntilEpochEnd.mul(1e18).div(1 days);
            uint256 multiplier = 1e18.add(2e18.mul(1e18.sub(timeRatio)).div(1e18)); // 1x to 3x cooldown
            baseCooldown = baseCooldown.mul(multiplier).div(1e18);
            
        } else if (timeUntilEpochEnd < 7 days) {
            // Last week: slight cooldown increase
            uint256 timeRatio = timeUntilEpochEnd.mul(1e18).div(7 days);
            uint256 multiplier = 1e18.add(1e18.mul(1e18.sub(timeRatio)).div(1e18)); // 1x to 2x cooldown
            baseCooldown = baseCooldown.mul(multiplier).div(1e18);
        }
        
        return baseCooldown > tMax ? tMax : baseCooldown;
    }
    
    /**
     * @dev Calculate grief multiplier for epoch-weighted transfers
     * @param timeUntilEpochEnd Time until epoch ends in seconds
     * @return multiplier Grief multiplier
     */
    function calculateGriefMultiplier(uint256 timeUntilEpochEnd) public view returns (uint256) {
        if (timeUntilEpochEnd == 0) return 1;
        
        uint256 epochDuration = EPOCH_DURATION;
        uint256 t = timeUntilEpochEnd;
        
        // grief_multiplier(t) = (T / t)^k
        uint256 ratio = epochDuration.mul(1e18).div(t);
        uint256 multiplier = ratio.pow(GRIEF_TAX_EXPONENT).div(1e18.pow(GRIEF_TAX_EXPONENT - 1));
        
        return multiplier > MAX_VIRTUAL_SIZE ? MAX_VIRTUAL_SIZE : multiplier;
    }
    
    /**
     * @dev Get minimum stake required
     * @return Minimum stake amount
     */
    function getMinimumStake() public view returns (uint256) {
        return totalSupply.mul(SYBIL_STAKE_PERCENTAGE).div(1000000); // 0.05%
    }
    
    /**
     * @dev Get current epoch time remaining
     * @return Time remaining in current epoch
     */
    function getEpochTimeRemaining() public view returns (uint256) {
        uint256 epochEndTime = epochStartTime.add(EPOCH_DURATION);
        if (block.timestamp >= epochEndTime) return 0;
        return epochEndTime.sub(block.timestamp);
    }
    
    /**
     * @dev Get user's current balance with demurrage applied
     * @param user Address to check
     * @return Current balance after demurrage
     */
    function getCurrentBalance(address user) external returns (uint256) {
        applyDemurrage(user);
        return balanceOf(user);
    }
    
    /**
     * @dev Bridge mint function (only callable by bridge)
     * @param to Recipient address
     * @param amount Amount to mint
     */
    function bridgeMint(address to, uint256 amount) external onlyBridge {
        _mint(to, amount);
    }
    
    /**
     * @dev Bridge burn function (only callable by bridge)
     * @param from Address to burn from
     * @param amount Amount to burn
     */
    function bridgeBurn(address from, uint256 amount) external onlyBridge {
        _burn(from, amount);
    }
    
    /**
     * @dev Finalize current epoch
     */
    function finalizeEpoch() external nonReentrant {
        require(block.timestamp >= epochStartTime.add(EPOCH_DURATION), "Epoch not over yet");
        require(!isEpochFinalized[currentEpoch], "Epoch already finalized");
        
        // Mark epoch as finalized
        isEpochFinalized[currentEpoch] = true;
        
        // Increment epoch
        currentEpoch = currentEpoch.add(1);
        epochStartTime = block.timestamp;
        
        emit EpochFinalized(currentEpoch.sub(1), msg.sender);
    }
    
    /**
     * @dev Get fee pot balance
     * @return Current fee pot balance
     */
    function getFeePot() external view returns (uint256) {
        return feePot;
    }

    /**
     * @dev Steal DUMP tokens from another player
     * @param victim Address to steal from
     * @param amount Amount to steal
     */
    function stealDump(address victim, uint256 amount) external nonReentrant onlyActiveParticipant {
        require(victim != address(0), "Cannot steal from zero address");
        require(victim != msg.sender, "Cannot steal from yourself");
        require(amount > 0, "Cannot steal zero amount");
        require(isActiveParticipant[victim], "Can only steal from active participants");
        
        // Apply demurrage to both thief and victim
        applyDemurrage(msg.sender);
        applyDemurrage(victim);
        
        // Check theft cooldown
        require(block.timestamp >= theftCooldownEndTime[msg.sender], "Theft cooldown active");
        
        // Check victim has enough balance
        uint256 victimBalance = balanceOf(victim);
        require(victimBalance >= amount, "Victim has insufficient balance");
        
        // Calculate theft fee (same as transfer fee)
        uint256 feeAmount = amount.mul(THEFT_FEE_BASIS_POINTS).div(DEMURRAGE_BASIS_POINTS);
        uint256 theftAmount = amount.sub(feeAmount);
        
        // Calculate theft cooldown (amount-scaled like transfers)
        uint256 cooldown = computeTheftCooldown(amount);
        
        // Calculate epoch-weighted theft cost (increases dramatically near snapshot)
        uint256 theftCost = calculateTheftCost(amount);
        
        // Check thief has enough balance to pay the theft cost
        uint256 thiefBalance = balanceOf(msg.sender);
        require(thiefBalance >= theftCost, "Insufficient balance to pay theft cost");
        
        // Update theft cooldown
        theftCooldownEndTime[msg.sender] = block.timestamp.add(cooldown);
        
        // Execute the theft
        _transfer(victim, msg.sender, theftAmount);
        
        // Pay theft cost (this makes thief "richer" = worse ranking)
        _burn(msg.sender, theftCost);
        
        // Collect fee
        if (feeAmount > 0) {
            _transfer(victim, address(this), feeAmount);
            feePot = feePot.add(feeAmount);
            emit FeeCollected(feeAmount, feePot);
        }
        
        emit TheftExecuted(msg.sender, victim, amount, feeAmount, cooldown);
    }
    
    /**
     * @dev Compute theft cooldown based on amount stolen
     * @param amount Amount being stolen
     * @return cooldown Cooldown duration in seconds
     */
    function computeTheftCooldown(uint256 amount) public view returns (uint256) {
        uint256 tMin = THEFT_COOLDOWN_MIN;
        uint256 tMax = THEFT_COOLDOWN_MAX;
        uint256 k = 2; // Quadratic scaling
        
        if (amount == 0) return tMin;
        
        uint256 scaled = amount.mul(1e18).div(totalSupply);
        uint256 cooldown = tMin.add(
            tMax.sub(tMin).mul(scaled.pow(k)).div(1e18.pow(k))
        );
        
        return cooldown > tMax ? tMax : cooldown;
    }
    
    /**
     * @dev Calculate theft cost based on epoch timing (continuously increases throughout epoch)
     * @param amount Amount being stolen
     * @return cost Theft cost in DUMP tokens
     */
    function calculateTheftCost(uint256 amount) public view returns (uint256) {
        uint256 timeUntilEpochEnd = getEpochTimeRemaining();
        uint256 epochDuration = EPOCH_DURATION;
        
        // Base cost is 5% of amount stolen (cheapest on day 1)
        uint256 baseCost = amount.mul(500).div(DEMURRAGE_BASIS_POINTS); // 5%
        
        // Calculate how much of the epoch has passed (0 = start, 1 = end)
        uint256 epochProgress = epochDuration.sub(timeUntilEpochEnd).mul(1e18).div(epochDuration);
        
        // Cost increases throughout the epoch with different phases:
        
        if (timeUntilEpochEnd < 1 hours) {
            // Last hour: EXPONENTIAL increase (meme territory)
            uint256 timeRatio = timeUntilEpochEnd.mul(1e18).div(1 hours);
            uint256 multiplier = 1e18.mul(1e18).div(timeRatio.pow(3)); // Cubic increase for maximum chaos
            
            // Cap at 1000x cost (literally losing money)
            if (multiplier > 1000e18) {
                multiplier = 1000e18;
            }
            
            baseCost = baseCost.mul(multiplier).div(1e18);
            
        } else if (timeUntilEpochEnd < 1 days) {
            // Last day: QUADRATIC increase (high risk)
            uint256 timeRatio = timeUntilEpochEnd.mul(1e18).div(1 days);
            uint256 multiplier = 1e18.add(19e18.mul(1e18.sub(timeRatio.pow(2))).div(1e18)); // 1x to 20x cost
            baseCost = baseCost.mul(multiplier).div(1e18);
            
        } else if (timeUntilEpochEnd < 7 days) {
            // Last week: LINEAR increase (moderate risk)
            uint256 timeRatio = timeUntilEpochEnd.mul(1e18).div(7 days);
            uint256 multiplier = 1e18.add(4e18.mul(1e18.sub(timeRatio)).div(1e18)); // 1x to 5x cost
            baseCost = baseCost.mul(multiplier).div(1e18);
            
        } else {
            // First 23 days: GRADUAL increase (low risk)
            // Use a smooth curve that starts at 1x and gradually increases
            uint256 timeRatio = timeUntilEpochEnd.mul(1e18).div(epochDuration);
            uint256 multiplier = 1e18.add(2e18.mul(1e18.sub(timeRatio)).div(1e18)); // 1x to 3x cost over 23 days
            baseCost = baseCost.mul(multiplier).div(1e18);
        }
        
        return baseCost;
    }
    
    /**
     * @dev Get theft cooldown status for an address
     * @param user Address to check
     * @return isOnCooldown Whether user is on theft cooldown
     * @return timeRemaining Time remaining on cooldown
     */
    function getTheftCooldownStatus(address user) external view returns (bool isOnCooldown, uint256 timeRemaining) {
        if (block.timestamp >= theftCooldownEndTime[user]) {
            return (false, 0);
        } else {
            return (true, theftCooldownEndTime[user].sub(block.timestamp));
        }
    }
}