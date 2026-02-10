/**
 * AgentMail Registry Client
 * 
 * Utilities for interacting with the AgentMail on-chain registry.
 */

import { 
  type Address, 
  type Rpc,
  findProgramDerivedAddress,
  encoding,
} from '@solana/kit';
import { fetchMaybeAgentRegistry, type AgentRegistry } from './generated/accounts/agentRegistry';
import { AGENTMAIL_PROGRAM_ADDRESS } from './generated/programs/agentmail';

/**
 * Derives the AgentRegistry PDA address for a given agent authority.
 * 
 * @param agentAuthority - The agent's public key
 * @param programId - The AgentMail program address (optional, defaults to generated program ID)
 * @returns The PDA address for the agent's registry
 */
export function findAgentRegistryPda(
  agentAuthority: Address,
  programId: Address = AGENTMAIL_PROGRAM_ADDRESS
): [Address, number] {
  return findProgramDerivedAddress({
    seeds: [
      encoding.getUtf8Encoder().encode('agentmail'),
      encoding.getBase58Encoder().encode(agentAuthority),
    ],
    programId,
  });
}

/**
 * Represents an agent's registry information with parsed string fields.
 */
export type ParsedAgentRegistry = {
  discriminator: number;
  version: number;
  bump: number;
  authority: Address;
  name: string;
  inboxUrl: string;
  createdAt: bigint;
  updatedAt: bigint;
};

/**
 * Parses the raw registry account data to extract string fields.
 * 
 * @param registry - Raw AgentRegistry account data
 * @returns Parsed registry with proper string fields
 */
export function parseAgentRegistry(registry: AgentRegistry): ParsedAgentRegistry {
  // Parse length-prefixed strings
  const nameBytes = registry.name;
  const nameLength = new DataView(new Uint8Array(nameBytes.slice(0, 4)).buffer).getUint32(0, true);
  const name = new TextDecoder().decode(new Uint8Array(nameBytes.slice(4, 4 + nameLength)));
  
  const inboxUrlBytes = registry.inboxUrl;
  const inboxUrlLength = new DataView(new Uint8Array(inboxUrlBytes.slice(0, 4)).buffer).getUint32(0, true);
  const inboxUrl = new TextDecoder().decode(new Uint8Array(inboxUrlBytes.slice(4, 4 + inboxUrlLength)));

  return {
    discriminator: registry.discriminator,
    version: registry.version,
    bump: registry.bump,
    authority: registry.authority,
    name,
    inboxUrl,
    createdAt: registry.createdAt,
    updatedAt: registry.updatedAt,
  };
}

/**
 * Looks up an agent's registry information by their authority (public key).
 * 
 * @param rpc - Solana RPC client
 * @param agentAuthority - The agent's public key
 * @param programId - The AgentMail program address (optional)
 * @returns The agent's registry information or null if not found
 */
export async function lookupAgentRegistry(
  rpc: Rpc,
  agentAuthority: Address,
  programId: Address = AGENTMAIL_PROGRAM_ADDRESS
): Promise<ParsedAgentRegistry | null> {
  try {
    // Derive the PDA address
    const [registryPda] = findAgentRegistryPda(agentAuthority, programId);
    
    // Fetch the account data
    const maybeAccount = await fetchMaybeAgentRegistry(rpc, registryPda);
    
    if (!maybeAccount.exists) {
      return null;
    }
    
    // Parse and return the registry data
    return parseAgentRegistry(maybeAccount.data);
  } catch (error) {
    console.error('Error looking up agent registry:', error);
    return null;
  }
}

/**
 * Gets an agent's inbox URL by their authority (public key).
 * 
 * @param rpc - Solana RPC client
 * @param agentAuthority - The agent's public key
 * @param programId - The AgentMail program address (optional)
 * @returns The agent's inbox URL or null if not found
 */
export async function getAgentInboxUrl(
  rpc: Rpc,
  agentAuthority: Address,
  programId: Address = AGENTMAIL_PROGRAM_ADDRESS
): Promise<string | null> {
  const registry = await lookupAgentRegistry(rpc, agentAuthority, programId);
  return registry?.inboxUrl ?? null;
}