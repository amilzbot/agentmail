/**
 * AgentMail TypeScript Client
 * 
 * Decentralized agent-to-agent messaging protocol on Solana.
 */

// Export generated client code
export * from './generated';

// Export custom utilities
export * from './registry';
export * from './messaging';

// Re-export commonly used types and functions for convenience
export type { ParsedAgentRegistry } from './registry';
export { 
  lookupAgentRegistry, 
  getAgentInboxUrl, 
  findAgentRegistryPda,
  parseAgentRegistry
} from './registry';

export type { 
  AgentMailMessage, 
  CreateMessageOptions, 
  SignedAgentMailMessage 
} from './messaging';
export { 
  createAgentMailMessage, 
  signAgentMailMessage, 
  createAndSignMessage, 
  verifyAgentMailMessage,
  serializeSignedMessage,
  deserializeSignedMessage
} from './messaging';