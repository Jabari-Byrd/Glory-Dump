const configuredProgram = import.meta.env.VITE_PROGRAM_ID?.trim() ?? "";
const configuredRpc = import.meta.env.VITE_SOLANA_RPC_URL?.trim() ?? "";

export const appConfig = Object.freeze({
  programId: configuredProgram,
  rpcUrl: configuredRpc || "https://api.devnet.solana.com",
  clusterLabel: configuredRpc ? "Configured Solana cluster" : "Solana Devnet",
  explorerBaseUrl: "https://explorer.solana.com",
  liveEnabled: configuredProgram.length > 0,
});
