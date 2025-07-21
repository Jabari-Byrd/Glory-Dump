// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./DumpToken.sol";

/**
 * @title BridgeGatekeeper
 * @dev Enforces cooldowns and rules for cross-chain DUMP transfers
 * @author Prime Anomaly
 */
contract BridgeGatekeeper is ReentrancyGuard {

    // Constants
    uint256 public constant MAX_TRANSFER_PER_EPOCH = 1000000 * 10**18; // 1M DUMP per epoch
    uint256 public constant COOLDOWN_PERIOD = 1 hours; // 1 hour cooldown between bridge transfers
    
    // State variables
    DumpToken public dumpToken;
    address public bridgeContract;
    address public owner;
    
    // Transfer tracking
    mapping(address => uint256) public lastBridgeTransfer;
    mapping(address => uint256) public epochTransferAmount;
    mapping(uint256 => uint256) public epochTotalTransfers;
    
    // Circuit breaker
    bool public bridgePaused;
    
    // Events
    event BridgeTransfer(address indexed from, address indexed to, uint256 amount, uint256 epoch);
    event BridgePaused(address indexed pauser, string reason);
    event BridgeUnpaused(address indexed unpauser);
    event BridgeContractUpdated(address indexed oldBridge, address indexed newBridge);
    
    // Modifiers
    modifier onlyBridge() {
        require(msg.sender == bridgeContract, "Only bridge can call");
        _;
    }
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call");
        _;
    }
    
    modifier whenNotPaused() {
        require(!bridgePaused, "Bridge is paused");
        _;
    }
    
    constructor(address _dumpToken, address _bridgeContract) {
        dumpToken = DumpToken(_dumpToken);
        bridgeContract = _bridgeContract;
        owner = msg.sender;
    }
    
    /**
     * @dev Validate and process bridge mint
     * @param to Recipient address
     * @param amount Amount to mint
     */
    function validateMint(address to, uint256 amount) external onlyBridge whenNotPaused {
        require(to != address(0), "Cannot mint to zero address");
        require(amount > 0, "Cannot mint zero amount");
        
        // Check cooldown
        require(block.timestamp >= lastBridgeTransfer[to] + COOLDOWN_PERIOD, "Cooldown active");
        
        // Check epoch limits
        uint256 currentEpoch = dumpToken.currentEpoch();
        require(epochTransferAmount[to] + amount <= MAX_TRANSFER_PER_EPOCH, "Epoch limit exceeded");
        
        // Check global epoch limit
        require(epochTotalTransfers[currentEpoch] + amount <= MAX_TRANSFER_PER_EPOCH * 100, "Global epoch limit exceeded");
        
        // Update tracking
        lastBridgeTransfer[to] = block.timestamp;
        epochTransferAmount[to] = epochTransferAmount[to] + amount;
        epochTotalTransfers[currentEpoch] = epochTotalTransfers[currentEpoch] + amount;
        
        // Mint tokens
        dumpToken.bridgeMint(to, amount);
        
        emit BridgeTransfer(address(0), to, amount, currentEpoch);
    }
    
    /**
     * @dev Validate and process bridge burn
     * @param from Address to burn from
     * @param amount Amount to burn
     */
    function validateBurn(address from, uint256 amount) external onlyBridge whenNotPaused {
        require(from != address(0), "Cannot burn from zero address");
        require(amount > 0, "Cannot burn zero amount");
        
        // Check cooldown
        require(block.timestamp >= lastBridgeTransfer[from] + COOLDOWN_PERIOD, "Cooldown active");
        
        // Check epoch limits
        uint256 currentEpoch = dumpToken.currentEpoch();
        require(epochTransferAmount[from] + amount <= MAX_TRANSFER_PER_EPOCH, "Epoch limit exceeded");
        
        // Check global epoch limit
        require(epochTotalTransfers[currentEpoch] + amount <= MAX_TRANSFER_PER_EPOCH * 100, "Global epoch limit exceeded");
        
        // Update tracking
        lastBridgeTransfer[from] = block.timestamp;
        epochTransferAmount[from] = epochTransferAmount[from] + amount;
        epochTotalTransfers[currentEpoch] = epochTotalTransfers[currentEpoch] + amount;
        
        // Burn tokens
        dumpToken.bridgeBurn(from, amount);
        
        emit BridgeTransfer(from, address(0), amount, currentEpoch);
    }
    
    /**
     * @dev Emergency pause bridge
     * @param reason Reason for pause
     */
    function emergencyPauseBridge(string memory reason) external onlyOwner {
        bridgePaused = true;
        emit BridgePaused(msg.sender, reason);
    }
    
    /**
     * @dev Emergency unpause bridge
     */
    function emergencyUnpauseBridge() external onlyOwner {
        bridgePaused = false;
        emit BridgeUnpaused(msg.sender);
    }
    
    /**
     * @dev Update bridge contract address
     * @param newBridge New bridge contract address
     */
    function updateBridgeContract(address newBridge) external onlyOwner {
        require(newBridge != address(0), "Invalid bridge address");
        address oldBridge = bridgeContract;
        bridgeContract = newBridge;
        emit BridgeContractUpdated(oldBridge, newBridge);
    }
    
    /**
     * @dev Transfer ownership
     * @param newOwner New owner address
     */
    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Invalid owner address");
        owner = newOwner;
    }
    
    /**
     * @dev Get user's transfer statistics
     * @param user Address to check
     * @return lastTransfer Last transfer timestamp
     * @return epochAmount Amount transferred in current epoch
     * @return cooldownEnd When cooldown ends
     */
    function getUserTransferStats(address user) external view returns (uint256, uint256, uint256) {
        uint256 lastTransfer = lastBridgeTransfer[user];
        uint256 epochAmount = epochTransferAmount[user];
        uint256 cooldownEnd = lastTransfer + COOLDOWN_PERIOD;
        
        return (lastTransfer, epochAmount, cooldownEnd);
    }
    
    /**
     * @dev Get epoch transfer statistics
     * @param epoch Epoch number to check
     * @return totalTransfers Total transfers in epoch
     */
    function getEpochTransferStats(uint256 epoch) external view returns (uint256) {
        return epochTotalTransfers[epoch];
    }
    
    /**
     * @dev Check if user can transfer
     * @param user Address to check
     * @param amount Amount to transfer
     * @return True if transfer is allowed
     */
    function canTransfer(address user, uint256 amount) external view returns (bool) {
        if (bridgePaused) return false;
        if (block.timestamp < lastBridgeTransfer[user] + COOLDOWN_PERIOD) return false;
        
        uint256 currentEpoch = dumpToken.currentEpoch();
        if (epochTransferAmount[user] + amount > MAX_TRANSFER_PER_EPOCH) return false;
        if (epochTotalTransfers[currentEpoch] + amount > MAX_TRANSFER_PER_EPOCH * 100) return false;
        
        return true;
    }
    
    /**
     * @dev Reset epoch tracking (called at epoch end)
     */
    function resetEpochTracking() external {
        // This should be called by the epoch finalization process
        // to reset per-user epoch amounts
        // TODO: Implement epoch reset logic
    }
}