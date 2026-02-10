/**
 * Register Nix as the first agent on AgentMail.
 */
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  address,
  createKeyPairSignerFromBytes,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  sendAndConfirmTransactionFactory,
  getAddressEncoder,
  getUtf8Encoder,
  getProgramDerivedAddress,
  type IInstruction,
  AccountRole,
} from '@solana/kit';
import { readFileSync } from 'fs';

const PROGRAM_ID = address('AMz2ybwRihFL9X4igLBtqNBEe9qqb4yUvjwNwEaPjNiX');
const SYSTEM_PROGRAM = address('11111111111111111111111111111111');
const RPC_URL = 'https://api.devnet.solana.com';
const WSS_URL = 'wss://api.devnet.solana.com';

function encodeString(s: string): Uint8Array {
  const bytes = new TextEncoder().encode(s);
  const buf = new ArrayBuffer(4 + bytes.length);
  new DataView(buf).setUint32(0, bytes.length, true);
  new Uint8Array(buf).set(bytes, 4);
  return new Uint8Array(buf);
}

async function main() {
  const keyBytes = JSON.parse(readFileSync('/home/node/.openclaw/workspace/.keys/nix.json', 'utf-8'));
  const signer = await createKeyPairSignerFromBytes(new Uint8Array(keyBytes));
  console.log('Nix address:', signer.address);

  const [registryPda, bump] = await getProgramDerivedAddress({
    seeds: [
      getUtf8Encoder().encode('agentmail'),
      getAddressEncoder().encode(signer.address),
    ],
    programAddress: PROGRAM_ID,
  });
  console.log('Registry PDA:', registryPda);
  console.log('Bump:', bump);

  // Instruction data: discriminator(3) + bump + name + inbox_url
  const nameBytes = encodeString('Nix');
  const inboxBytes = encodeString('https://nix.agentmail.dev/inbox');
  const data = new Uint8Array(1 + 1 + nameBytes.length + inboxBytes.length);
  data[0] = 3;
  data[1] = bump;
  data.set(nameBytes, 2);
  data.set(inboxBytes, 2 + nameBytes.length);

  const ix: IInstruction = {
    programAddress: PROGRAM_ID,
    accounts: [
      { address: signer.address, role: AccountRole.WRITABLE_SIGNER, signer },          // payer
      { address: signer.address, role: AccountRole.READONLY_SIGNER, signer },           // agent_authority
      { address: registryPda, role: AccountRole.WRITABLE },                              // agent_registry
      { address: SYSTEM_PROGRAM, role: AccountRole.READONLY },                           // system_program
      { address: PROGRAM_ID, role: AccountRole.READONLY },                               // agentmail_program
    ],
    data,
  };

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
  const sig = await sendAndConfirm(signedTx, { commitment: 'confirmed' });
  
  console.log('✅ Nix registered as first AgentMail user!');
  console.log('TX:', sig);
  console.log(`Explorer: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
}

main().catch(err => {
  console.error('Failed:', err);
  process.exit(1);
});
