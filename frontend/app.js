// GLORY/DUMP Solana Frontend Application
class GloryDumpApp {
    constructor() {
        this.connection = null;
        this.wallet = null;
        this.program = null;
        this.gameState = null;
        this.playerState = null;
        this.isConnected = false;

        // Program ID (update after deployment)
        this.programId = new solanaWeb3.PublicKey('GDgame1111111111111111111111111111111111111');
        
        // Network endpoint
        this.endpoint = 'https://api.devnet.solana.com'; // Change for mainnet: 'https://api.mainnet-beta.solana.com'

        this.init();
    }

    async init() {
        this.connection = new solanaWeb3.Connection(this.endpoint, 'confirmed');
        this.setupEventListeners();
        this.updateUI();

        // Check if Phantom wallet is installed and connected
        if (window.solana && window.solana.isPhantom) {
            try {
                const response = await window.solana.connect({ onlyIfTrusted: true });
                if (response.publicKey) {
                    this.wallet = window.solana;
                    this.isConnected = true;
                    await this.loadProgramData();
                    this.updateUI();
                }
            } catch (err) {
                console.log('Wallet not auto-connected');
            }
        }
    }

    setupEventListeners() {
        // Wallet connection
        document.getElementById('connectWallet').addEventListener('click', () => {
            this.connectWallet();
        });

        // Staking
        document.getElementById('stakeButton').addEventListener('click', () => {
            this.stakeForParticipation();
        });

        document.getElementById('withdrawStakeButton').addEventListener('click', () => {
            this.withdrawStake();
        });

        // Epoch signup
        document.getElementById('signupButton').addEventListener('click', () => {
            this.signUpForEpoch();
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

        // Theft slider
        const theftSlider = document.getElementById('theftSlider');
        theftSlider.addEventListener('input', (e) => {
            this.updateTheftAmount(e.target.value);
        });

    // Theft button
    document.getElementById('stealButton').addEventListener('click', () => {
            this.executeTheft();
        });

        // Finalize epoch
        document.getElementById('finalizeEpoch').addEventListener('click', () => {
            this.finalizeEpoch();
        });

        // Update player average
        setInterval(() => {
            if (this.isConnected) {
                this.updatePlayerAverage();
            }
        }, 30000); // Update every 30 seconds
    }

    async connectWallet() {
        if (!window.solana || !window.solana.isPhantom) {
            alert('Please install Phantom wallet!');
            window.open('https://phantom.app/', '_blank');
            return;
        }

        try {
            const response = await window.solana.connect();
            this.wallet = window.solana;
            this.isConnected = true;
            
            await this.loadProgramData();
            this.updateUI();
            
            console.log('Connected to wallet:', response.publicKey.toString());
        } catch (err) {
            console.error('Failed to connect wallet:', err);
            alert('Failed to connect wallet');
        }
    }

    async loadProgramData() {
        if (!this.isConnected) return;

        try {
            // Load game state
            const [gameStatePda] = await solanaWeb3.PublicKey.findProgramAddress(
                [Buffer.from('game_state')],
                this.programId
            );

            const gameStateInfo = await this.connection.getAccountInfo(gameStatePda);
            if (gameStateInfo) {
                this.gameState = gameStatePda;
                // In a real app, you'd deserialize the account data here
                console.log('Game state loaded:', gameStatePda.toString());
            }

            // Load player state
            const [playerStatePda] = await solanaWeb3.PublicKey.findProgramAddress(
                [Buffer.from('player_state'), this.wallet.publicKey.toBuffer()],
                this.programId
            );

            const playerStateInfo = await this.connection.getAccountInfo(playerStatePda);
            if (playerStateInfo) {
                this.playerState = playerStatePda;
                console.log('Player state loaded:', playerStatePda.toString());
            }

            await this.updateBalances();
            await this.updateEpochInfo();
        } catch (err) {
            console.error('Failed to load program data:', err);
        }
    }

    async updateBalances() {
        if (!this.isConnected) return;

        try {
            // In a real implementation, you would:
            // 1. Get DUMP token balance from player's associated token account
            // 2. Get GLORY token balance
            // 3. Get player state data (stake, cooldowns, etc.)
            // 4. Get current epoch information
            
            // For now, show placeholder data
            document.getElementById('dumpBalance').textContent = '0 DUMP';
            document.getElementById('gloryBalance').textContent = '0 GLORY';
            document.getElementById('stakeAmount').textContent = '0 DUMP';
            document.getElementById('userRank').textContent = '-';
            
            // Update cooldown displays
            document.getElementById('giveCooldownTime').textContent = '0s';
            document.getElementById('takeCooldownTime').textContent = '0s';
            
        } catch (err) {
            console.error('Failed to update balances:', err);
        }
    }

    async updateEpochInfo() {
        try {
            // In a real implementation, fetch from game state account
            document.getElementById('currentEpoch').textContent = '1';
            document.getElementById('epochTimeRemaining').textContent = '30 days remaining';
            document.getElementById('epochPhase').textContent = 'Waiting Period';
            
            // Update join fee based on time in waiting period
            document.getElementById('currentJoinFee').textContent = '0.01 SOL';
            
        } catch (err) {
            console.error('Failed to update epoch info:', err);
        }
    }

    async stakeForParticipation() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }

        const amountInput = document.getElementById('stakeAmountInput');
        const amount = parseInt(amountInput.value);

        if (!amount || amount < 1000000) {
            alert('Please enter a valid stake amount (minimum 1,000,000 DUMP)');
            return;
        }

        try {
            // In a real implementation, you would:
            // 1. Check if player has enough DUMP tokens
            // 2. Create the stake transaction
            // 3. Sign and send the transaction
            // 4. Wait for confirmation and update UI
            
            console.log('Staking', amount, 'DUMP tokens for participation');
            alert('Staking functionality will be implemented with the full Anchor setup');
            
        } catch (err) {
            console.error('Staking failed:', err);
            alert('Staking failed: ' + err.message);
        }
    }

    async withdrawStake() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }

        try {
            const confirmed = confirm('Withdraw your staked DUMP tokens? This will remove you from active participation.');
            if (!confirmed) return;

            // In a real implementation, create and send withdraw transaction
            console.log('Withdrawing staked DUMP tokens');
            alert('Stake withdrawal functionality will be implemented with the full Anchor setup');
            
        } catch (err) {
            console.error('Stake withdrawal failed:', err);
            alert('Withdrawal failed: ' + err.message);
        }
    }

    async signUpForEpoch() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }

        try {
            // Calculate current join fee based on time in waiting period
            const joinFee = 0.01; // SOL - this would be calculated dynamically
            
            const confirmed = confirm(`Sign up for next epoch for ${joinFee} SOL?`);
            if (!confirmed) return;

            // In a real implementation, you would create and send the signup transaction
            console.log('Signing up for next epoch with fee:', joinFee, 'SOL');
            alert('Epoch signup functionality will be implemented with the full Anchor setup');
            
        } catch (err) {
            console.error('Epoch signup failed:', err);
            alert('Signup failed: ' + err.message);
        }
    }

    async executeDump() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }

        const target = document.getElementById('dumpTarget').value;
        const amount = parseInt(document.getElementById('dumpAmount').textContent.replace(/,/g, ''));

        if (!target || !amount) {
            alert('Please enter a valid target address and amount');
            return;
        }

        try {
            // Validate Solana address
            new solanaWeb3.PublicKey(target);
            
            console.log('Transferring', amount, 'DUMP to', target);
            
            // In a real implementation, create and send transfer transaction
            alert('DUMP transfer functionality will be implemented with the full Anchor setup');
            
        } catch (err) {
            console.error('Transfer failed:', err);
            alert('Transfer failed: ' + err.message);
        }
    }

    async executeTheft() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }

        const target = document.getElementById('theftTarget').value;
        const amount = parseInt(document.getElementById('theftAmount').textContent.replace(/,/g, ''));

        if (!target || !amount) {
            alert('Please enter a valid target address and amount');
            return;
        }

        try {
            // Validate Solana address
            new solanaWeb3.PublicKey(target);
            
            console.log('Attempted theft', amount, 'DUMP from', target);
            alert('Theft is temporarily disabled pending secure implementation.');
            
        } catch (err) {
            console.error('Theft failed:', err);
            alert('Theft failed: ' + err.message);
        }
    }

    async finalizeEpoch() {
        if (!this.isConnected) {
            alert('Please connect your wallet first');
            return;
        }

        try {
            console.log('Finalizing current epoch');
            
            // In a real implementation, create and send finalization transaction
            alert('Epoch finalization functionality will be implemented with the full Anchor setup');
            
        } catch (err) {
            console.error('Epoch finalization failed:', err);
            alert('Finalization failed: ' + err.message);
        }
    }

    async updatePlayerAverage() {
        if (!this.isConnected) return;

        try {
            // In a real implementation, call the update_player_average instruction
            console.log('Updating time-weighted average...');
        } catch (err) {
            console.error('Failed to update player average:', err);
        }
    }

    updateDumpAmount(sliderValue) {
        // Convert slider value to actual DUMP amount
        const maxDump = 1000000; // This would come from player's actual balance
        const amount = Math.floor((sliderValue / 100) * maxDump);
        document.getElementById('dumpAmount').textContent = amount.toLocaleString();
        
        // Update fee display (0.3%)
        const fee = Math.floor(amount * 0.003);
        document.getElementById('transferFee').textContent = `${fee.toLocaleString()} DUMP`;
    }

    updateTheftAmount(sliderValue) {
        // Convert slider value to actual DUMP amount
        const maxDump = 1000000; // This would come from target's actual balance
        const amount = Math.floor((sliderValue / 100) * maxDump);
        document.getElementById('theftAmount').textContent = amount.toLocaleString();
    }

    updateUI() {
        const connectButton = document.getElementById('connectWallet');
        const walletAddress = document.getElementById('walletAddress');

        if (this.isConnected && this.wallet) {
            connectButton.textContent = 'Connected';
            connectButton.disabled = true;
            connectButton.classList.add('connected');
            walletAddress.textContent = this.wallet.publicKey.toString().slice(0, 8) + '...';
        } else {
            connectButton.textContent = 'Connect Phantom';
            connectButton.disabled = false;
            connectButton.classList.remove('connected');
            walletAddress.textContent = '';
        }
    }

    // Helper method to format large numbers
    formatNumber(num) {
        if (num >= 1e9) return (num / 1e9).toFixed(2) + 'B';
        if (num >= 1e6) return (num / 1e6).toFixed(2) + 'M';
        if (num >= 1e3) return (num / 1e3).toFixed(2) + 'K';
        return num.toString();
    }
}

// Initialize the app when the page loads
document.addEventListener('DOMContentLoaded', () => {
    window.gloryDumpApp = new GloryDumpApp();
});