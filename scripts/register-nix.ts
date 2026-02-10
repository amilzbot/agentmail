/**
 * Register an agent on AgentMail — demonstrates using the client library.
 *
 * Usage:
 *   WALLET=~/.config/solana/id.json bun run scripts/register-nix.ts
 *   bun run scripts/register-nix.ts --wallet ./my-key.json
 */
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createKeyPairSignerFromBytes,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  sendAndConfirmTransactionFactory,
} from '@solana/kit';
import { readFileSync } from 'fs';
import { getRegisterAgentInstruction } from '../clients/typescript/src/generated/instructions/registerAgent';
import { AGENTMAIL_PROGRAM_ADDRESS } from '../clients/typescript/src/generated/programs/agentmail';
import { findAgentRegistryPda } from '../clients/typescript/src/registry';

const RPC_URL = 'https://api.devnet.solana.com';
const WSS_URL = 'wss://api.devnet.solana.com';

function getWalletPath(): string {
  const idx = process.argv.indexOf('--wallet');
  if (idx !== -1 && process.argv[idx + 1]) return process.argv[idx + 1];
  if (process.env.WALLET) return process.env.WALLET;

  console.error('Provide a wallet: --wallet <path> or WALLET=<path>');
  process.exit(1);
}

async function main() {
  const walletPath = getWalletPath();
  const keyBytes = JSON.parse(readFileSync(walletPath, 'utf-8'));
  const signer = await createKeyPairSignerFromBytes(new Uint8Array(keyBytes));
  console.log('Agent address:', signer.address);

  // Derive registry PDA
  const [registryPda, bump] = await findAgentRegistryPda(signer.address);
  console.log('Registry PDA:', registryPda);

  // Build instruction using the generated client
  const ix = getRegisterAgentInstruction({
    payer: signer,
    agentAuthority: signer,
    agentRegistry: registryPda,
    agentmailProgram: AGENTMAIL_PROGRAM_ADDRESS,
    bump,
    name: 'Nix',
    inboxUrl: 'https://nix.agentmail.dev/inbox',
  });

  const rpc = createSolanaRpc(RPC_URL);
  const rpcSubscriptions = createSolanaRpcSubscriptions(WSS_URL);
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  const txMessage = pipe(
    createTransactionMessage({ version: 0 }),
    tx => setTransactionMessageFeePayer(signer.address, tx),
    tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    tx => appendTransactionMessageInstruction(ix, tx),
  );

  const signedTx = await signTransactionMessageWithSigners(txMessage);
  const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

  console.log('Sending transaction...');
  const sig = await sendAndConfirm(signedTx as any, { commitment: 'confirmed' });

  console.log('Registered on AgentMail.');
  console.log('TX:', sig);
  console.log(`Explorer: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
}

main().catch(err => {
  console.error('Failed:', err);
  process.exit(1);
});
