# GLORY/DUMP deployment guide

This guide stops at localnet and Devnet. The v3 program is experimental and unaudited; mainnet deployment is outside the supported procedure.

## 1. Install the pinned toolchain

Use the versions recorded in `Anchor.toml` and `package.json`:

```text
Anchor CLI  1.2.0
Solana CLI  4.1.2
Node.js     >=22.12
pnpm        11.19.0
```

Then install dependencies and verify the host build:

```bash
pnpm install
pnpm run check
```

Do not treat that host build as an SBF result. A deployable release must also pass `anchor build` and validator-backed instruction tests under the pinned Solana toolchain.

## 2. Create and bind the program identity

The committed `Fg6Pa…` address is a development placeholder. Never deploy it as though the repository controls that key.

From a clean checkout:

```bash
anchor build
anchor keys sync
anchor build
pnpm run idl
anchor keys list
```

The first build creates a local program keypair under the ignored `target/deploy/` directory. `anchor keys sync` writes its public address into `declare_id!` and the cluster entries in `Anchor.toml`; the second build and IDL generation bind every artifact to that address.

Keep the generated program keypair private and backed up. Never commit it, paste it into an issue, or reuse a wallet that holds unrelated funds.

Before continuing, confirm these all match exactly:

- `anchor keys list`;
- `declare_id!` in `programs/glory_dump/src/lib.rs`;
- `programs.localnet` and `programs.devnet` in `Anchor.toml`;
- `address` in `frontend/src/idl/glory_dump.json`.

Any mismatch is a stop condition.

## 3. Local-validator rehearsal

Use a dedicated local wallet and keep the validator visible in another terminal:

```bash
solana config set --url localhost
solana-test-validator --reset
```

In the project terminal:

```bash
anchor build
anchor deploy
export SOLANA_WALLET_PATH=/absolute/path/to/local-validator-wallet.json
export SOLANA_RPC_URL=http://127.0.0.1:8899
pnpm run initialize:protocol
pnpm run smoke:localnet
```

The initializer creates exactly four fixed PDAs: the protocol, capped GLORY mint, epoch 1, and epoch-1 leaderboard. It refuses to run twice. The smoke command then registers its isolated wallet and verifies the exact participant increment, 0.002 SOL bond delta, commitment, owner, and four zeroed lane accounts. It also refuses to reuse an already registered smoke wallet. Record both commands' addresses and transaction signatures.

Configure the Strategy Room:

```bash
cp frontend/.env.example frontend/.env
```

Set `VITE_PROGRAM_ID` to the synchronized address and `VITE_SOLANA_RPC_URL` to `http://127.0.0.1:8899`, then run:

```bash
pnpm run dev
```

The wallet must be configured for the same local RPC. Complete the validator matrix in `docs/TESTING.md`; merely seeing the page connect is not enough.

## 4. Devnet rehearsal

Use a new deployment wallet that contains only Devnet SOL. Confirm the active URL and wallet before every command:

```bash
solana config set --url devnet
solana config get
solana balance
anchor deploy --provider.cluster devnet
```

Initialize against Devnet without putting the keypair or a private RPC URL in source control:

```bash
export SOLANA_WALLET_PATH=/absolute/path/to/devnet-deployer.json
export SOLANA_RPC_URL=https://api.devnet.solana.com
pnpm run initialize:protocol
```

The public Devnet endpoint is suitable for a small rehearsal, not a production indexer or high-volume game. Use a dedicated RPC provider for load tests and keep credentials in `frontend/.env` or deployment secrets, never in the committed IDL or HTML.

Record:

- clean Git commit and lockfile hash;
- Rust, Solana, Anchor, Node, and pnpm versions;
- SBF binary SHA-256 and generated IDL SHA-256;
- program ID, ProgramData address, and upgrade authority;
- deploy and initialize transaction signatures;
- protocol, GLORY mint, first epoch, and leaderboard addresses;
- frontend build hash and exact RPC cluster;
- all validator-test results and observed compute units.

## 5. Upgrade authority and immutability

The program has no administrator instruction, but a normal deployment is still upgradeable by its deployment authority. Those are different facts.

During audited Devnet iteration, store the upgrade authority in an explicitly documented multisignature or other reviewed custody arrangement. Before any claim of immutability, independently verify the deployed binary, IDL, configuration, and incident plan. Only then can the authority be removed:

```bash
solana program show PROGRAM_ID
solana program set-upgrade-authority PROGRAM_ID --final
solana program show PROGRAM_ID
```

`--final` is irreversible: the program can never be upgraded or closed afterward. Do not execute it merely to satisfy the project's “no admin keys” theme.

## 6. Frontend release

Build from the same clean commit used for deployment:

```bash
pnpm run check:frontend
pnpm run build
```

Deploy only `dist/`. Confirm the built client contains the expected program ID and RPC, loads in demo mode when configuration is absent, and never requests a wallet signature merely to render public state.

The live battle feed needs an independently operated event indexer. RPC-wide account scans are filtered by epoch, but transaction-history ingestion should not be pushed into every player's browser.

## Mainnet hold

There is no supported mainnet command. Mainnet requires all gates in `README.md`, `SECURITY.md`, and `docs/TESTING.md`, including an independent audit, economic simulations, SBF/validator evidence, randomness resolution, load testing, legal review, and a deliberate upgrade-authority decision.
