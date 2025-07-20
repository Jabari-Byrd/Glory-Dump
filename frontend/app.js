// DUMP/GLORY Frontend Application
class DumpGloryApp {
    constructor() {
        this.provider = null;
        this.signer = null;
        this.dumpToken = null;
        this.gloryToken = null;
        this.feePot = null;
        this.isConnected = false;
        
        // Contract addresses (update these after deployment)
        this.contractAddresses = {
            dumpToken: '0x...', // Update with deployed address
            gloryToken: '0x...', // Update with deployed address
            feePot: '0x...'      // Update with deployed address
        };
        
        this.init();
    }
    
    async init() {
        this.setupEventListeners();
        this.updateUI();
        
        // Check if wallet is already connected
        if (typeof window.ethereum !== 'undefined') {
            const accounts = await window.ethereum.request({ method: 'eth_accounts' });
            if (accounts.length > 0) {
                await this.connectWallet();
            }
        }
    }
    
    setupEventListeners() {
        // Wallet connection
        document.getElementById('connectWallet').addEventListener('click', () => {
            this.connectWallet();
        });
        
        // Dump slider
        const dumpSlider = document.getElementById('dumpSlider');
        dumpSlider.addEventListener('input', (e) => {
            this.updateDumpAmount(e.target.value);
        });
        
        // Dump button
        document.getElementById('dumpButton').addEventListener('click', () => {
            this.executeDump();
        });
        
        // Finalize epoch
        document.getElementById('finalizeEpoch').addEventListener('click', () => {
            this.finalizeEpoch();
        });
        
        // Execute buyback
        document.getElementById('executeBuyback').addEventListener('click', () => {
            this.executeBuyback();
        });
        
        // Wallet account changes
        if (typeof window.ethereum !== 'undefined') {
            window.ethereum.on('accountsChanged', (accounts) => {
                if (accounts.length === 0) {
                    this.disconnectWallet();
                } else {
                    this.connectWallet();
                }
            });
            
            window.ethereum.on('chainChanged', () => {
                window.location.reload();
            });
        }
    }
    
    async connectWallet() {
        try {
            if (typeof window.ethereum === 'undefined') {
                alert('Please install MetaMask or another Web3 wallet');
                return;
            }
            
            // Request account access
            const accounts = await window.ethereum.request({
                method: 'eth_requestAccounts'
            });
            
            if (accounts.length === 0) {
                throw new Error('No accounts found');
            }
            
            // Setup provider and signer
            this.provider = new ethers.providers.Web3Provider(window.ethereum);
            this.signer = this.provider.getSigner();
            
            // Initialize contracts
            await this.initializeContracts();
            
            this.isConnected = true;
            this.updateUI();
            this.loadGameData();
            
        } catch (error) {
            console.error('Error connecting wallet:', error);
            alert('Failed to connect wallet: ' + error.message);
        }
    }
    
    disconnectWallet() {
        this.provider = null;
        this.signer = null;
        this.dumpToken = null;
        this.gloryToken = null;
        this.feePot = null;
        this.isConnected = false;
        this.updateUI();
    }
    
    async initializeContracts() {
        // Contract ABIs (simplified for demo)
        const dumpTokenABI = [
            'function balanceOf(address) view returns (uint256)',
            'function transfer(address, uint256) returns (bool)',
            'function getCurrentBalance(address) returns (uint256)',
            'function getFeePot() view returns (uint256)',
            'function computeCooldown(uint256) view returns (uint256)',
            'function cooldownEndTime(address) view returns (uint256)',
            'function isActiveParticipant(address) view returns (bool)',
            'function stakeForParticipation(uint256)',
            'function currentEpoch() view returns (uint256)',
            'function getEpochTimeRemaining() view returns (uint256)'
        ];
        
        const gloryTokenABI = [
            'function balanceOf(address) view returns (uint256)',
            'function getUserRank(address) view returns (int256)',
            'function getLeaderboard() view returns (address[])',
            'function getUserAverageDumpHeld(address) view returns (uint256)',
            'function getEpochTimeRemaining() view returns (uint256)',
            'function currentEpoch() view returns (uint256)',
            'function finalizeEpoch()'
        ];
        
        const feePotABI = [
            'function totalFeesCollected() view returns (uint256)',
            'function totalGloryBurned() view returns (uint256)',
            'function executeBuyback()'
        ];
        
        // Initialize contract instances
        this.dumpToken = new ethers.Contract(
            this.contractAddresses.dumpToken,
            dumpTokenABI,
            this.signer
        );
        
        this.gloryToken = new ethers.Contract(
            this.contractAddresses.gloryToken,
            gloryTokenABI,
            this.signer
        );
        
        this.feePot = new ethers.Contract(
            this.contractAddresses.feePot,
            feePotABI,
            this.signer
        );
    }
    
    async loadGameData() {
        if (!this.isConnected) return;
        
        try {
            const address = await this.signer.getAddress();
            
            // Load balances
            await this.loadBalances(address);
            
            // Load epoch info
            await this.loadEpochInfo();
            
            // Load leaderboard
            await this.loadLeaderboard();
            
            // Load fee pot info
            await this.loadFeePotInfo();
            
        } catch (error) {
            console.error('Error loading game data:', error);
        }
    }
    
    async loadBalances(address) {
        try {
            // Get current DUMP balance (with demurrage applied)
            const dumpBalance = await this.dumpToken.getCurrentBalance(address);
            const gloryBalance = await this.gloryToken.balanceOf(address);
            const userRank = await this.gloryToken.getUserRank(address);
            
            // Update UI
            document.getElementById('dumpBalance').textContent = 
                this.formatTokenAmount(dumpBalance, 18) + ' DUMP';
            document.getElementById('gloryBalance').textContent = 
                this.formatTokenAmount(gloryBalance, 18) + ' GLORY';
            
            if (userRank >= 0) {
                document.getElementById('userRank').textContent = '#' + (userRank + 1);
            } else {
                document.getElementById('userRank').textContent = 'Not Participating';
            }
            
        } catch (error) {
            console.error('Error loading balances:', error);
        }
    }
    
    async loadEpochInfo() {
        try {
            const currentEpoch = await this.gloryToken.currentEpoch();
            const timeRemaining = await this.gloryToken.getEpochTimeRemaining();
            
            document.getElementById('currentEpoch').textContent = currentEpoch;
            document.getElementById('epochTimeRemaining').textContent = 
                this.formatTimeRemaining(timeRemaining);
            
            // Enable/disable finalize button
            const finalizeButton = document.getElementById('finalizeEpoch');
            finalizeButton.disabled = timeRemaining > 0;
            
        } catch (error) {
            console.error('Error loading epoch info:', error);
        }
    }
    
    async loadLeaderboard() {
        try {
            const leaderboard = await this.gloryToken.getLeaderboard();
            const leaderboardList = document.getElementById('leaderboardList');
            
            if (leaderboard.length === 0) {
                leaderboardList.innerHTML = '<div class="loading">No participants yet</div>';
                return;
            }
            
            let html = '';
            for (let i = 0; i < Math.min(leaderboard.length, 10); i++) {
                const address = leaderboard[i];
                const avgDumpHeld = await this.gloryToken.getUserAverageDumpHeld(address);
                
                html += `
                    <div class="leaderboard-item">
                        <span>#${i + 1}</span>
                        <span>${this.shortenAddress(address)}</span>
                        <span>${this.formatTokenAmount(avgDumpHeld, 18)} DUMP</span>
                    </div>
                `;
            }
            
            leaderboardList.innerHTML = html;
            
        } catch (error) {
            console.error('Error loading leaderboard:', error);
            document.getElementById('leaderboardList').innerHTML = 
                '<div class="loading">Error loading leaderboard</div>';
        }
    }
    
    async loadFeePotInfo() {
        try {
            const totalFees = await this.feePot.totalFeesCollected();
            const gloryBurned = await this.feePot.totalGloryBurned();
            
            document.getElementById('totalFees').textContent = 
                this.formatTokenAmount(totalFees, 18);
            document.getElementById('gloryBurned').textContent = 
                this.formatTokenAmount(gloryBurned, 18);
            
        } catch (error) {
            console.error('Error loading fee pot info:', error);
        }
    }
    
    updateDumpAmount(percentage) {
        if (!this.isConnected) return;
        
        // This is a simplified calculation - in reality, you'd get the user's actual balance
        const maxAmount = 1000; // Placeholder
        const amount = (maxAmount * percentage) / 100;
        
        document.getElementById('dumpAmount').textContent = 
            this.formatTokenAmount(ethers.utils.parseEther(amount.toString()), 18);
        
        // Update cooldown and fee info
        this.updateTransferInfo(amount);
    }
    
    async updateTransferInfo(amount) {
        try {
            const amountWei = ethers.utils.parseEther(amount.toString());
            const cooldown = await this.dumpToken.computeCooldown(amountWei);
            const fee = amountWei.mul(30).div(10000); // 0.3% fee
            
            document.getElementById('cooldownTime').textContent = 
                this.formatTimeRemaining(cooldown);
            document.getElementById('transferFee').textContent = 
                this.formatTokenAmount(fee, 18);
            
        } catch (error) {
            console.error('Error updating transfer info:', error);
        }
    }
    
    async executeDump() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }
        
        const target = document.getElementById('dumpTarget').value;
        const amount = document.getElementById('dumpAmount').textContent.split(' ')[0];
        
        if (!target || !ethers.utils.isAddress(target)) {
            alert('Please enter a valid target address');
            return;
        }
        
        if (parseFloat(amount) <= 0) {
            alert('Please select an amount to dump');
            return;
        }
        
        try {
            const amountWei = ethers.utils.parseEther(amount);
            const tx = await this.dumpToken.transfer(target, amountWei);
            
            alert('Dump transaction sent! Hash: ' + tx.hash);
            await tx.wait();
            
            // Reload data
            await this.loadGameData();
            
        } catch (error) {
            console.error('Error executing dump:', error);
            alert('Failed to execute dump: ' + error.message);
        }
    }
    
    async finalizeEpoch() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }
        
        try {
            const tx = await this.gloryToken.finalizeEpoch();
            alert('Epoch finalization transaction sent! Hash: ' + tx.hash);
            await tx.wait();
            
            // Reload data
            await this.loadGameData();
            
        } catch (error) {
            console.error('Error finalizing epoch:', error);
            alert('Failed to finalize epoch: ' + error.message);
        }
    }
    
    async executeBuyback() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }
        
        try {
            const tx = await this.feePot.executeBuyback();
            alert('Buyback transaction sent! Hash: ' + tx.hash);
            await tx.wait();
            
            // Reload data
            await this.loadGameData();
            
        } catch (error) {
            console.error('Error executing buyback:', error);
            alert('Failed to execute buyback: ' + error.message);
        }
    }
    
    updateUI() {
        const connectButton = document.getElementById('connectWallet');
        const walletAddress = document.getElementById('walletAddress');
        
        if (this.isConnected) {
            connectButton.textContent = 'Connected';
            connectButton.disabled = true;
            this.signer.getAddress().then(address => {
                walletAddress.textContent = this.shortenAddress(address);
            });
        } else {
            connectButton.textContent = 'Connect Wallet';
            connectButton.disabled = false;
            walletAddress.textContent = '';
        }
    }
    
    // Utility functions
    formatTokenAmount(amount, decimals) {
        return ethers.utils.formatUnits(amount, decimals);
    }
    
    formatTimeRemaining(seconds) {
        if (seconds === 0) return '0s';
        
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;
        
        if (days > 0) return `${days}d ${hours}h`;
        if (hours > 0) return `${hours}h ${minutes}m`;
        if (minutes > 0) return `${minutes}m ${secs}s`;
        return `${secs}s`;
    }
    
    shortenAddress(address) {
        return address.substring(0, 6) + '...' + address.substring(address.length - 4);
    }
}

// Initialize the app when the page loads
document.addEventListener('DOMContentLoaded', () => {
    window.dumpGloryApp = new DumpGloryApp();
});