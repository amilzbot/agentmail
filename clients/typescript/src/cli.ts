#!/usr/bin/env bun
/**
 * AgentMail CLI
 * 
 * Command-line interface for the AgentMail protocol
 */

import { Command } from 'commander';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';
import {
  generateKeyPair,
  createKeyPairFromBytes,
  getAddressFromPublicKey,
  type Address,
} from '@solana/kit';
import { 
  verifyAgentMailMessage,
  createSignAndSendMessage,
  type CreateMessageOptions,
} from './messaging.js';
import {
  lookupAgentRegistry,
  getAgentInboxUrl,
} from './registry.js';

// Configuration
const CONFIG_DIR = join(homedir(), '.agentmail');
const KEYPAIR_PATH = join(CONFIG_DIR, 'keypair.json');
const INBOX_HISTORY_PATH = join(CONFIG_DIR, 'inbox.jsonl');

// CLI Program
const program = new Command();
program
  .name('agentmail')
  .description('AgentMail - Decentralized agent-to-agent messaging')
  .version('1.0.0');

/**
 * Load or create keypair
 */
async function getOrCreateKeypair(): Promise<{ keyPair: CryptoKeyPair; address: Address }> {
  if (existsSync(KEYPAIR_PATH)) {
    try {
      const keyData = JSON.parse(readFileSync(KEYPAIR_PATH, 'utf8'));
      const keyPair = await createKeyPairFromBytes(new Uint8Array(keyData));
      const address = await getAddressFromPublicKey(keyPair.publicKey);
      return { keyPair, address };
    } catch (error) {
      console.error('Failed to load existing keypair:', error);
      process.exit(1);
    }
  }

  // Create new keypair
  console.log('Creating new keypair...');
  const keyPair = await generateKeyPair();
  const address = await getAddressFromPublicKey(keyPair.publicKey);
  
  // Export and save the keypair bytes
  try {
    const privateKeyBytes = new Uint8Array(await crypto.subtle.exportKey('pkcs8', keyPair.privateKey));
    const publicKeyBytes = new Uint8Array(await crypto.subtle.exportKey('raw', keyPair.publicKey));
    // Store as 64-byte array: 32 private + 32 public (Solana convention)
    const combined = new Uint8Array(64);
    // pkcs8 wraps the raw key; for Ed25519 the raw 32 bytes are at the end
    combined.set(privateKeyBytes.slice(-32), 0);
    combined.set(publicKeyBytes, 32);
    writeFileSync(KEYPAIR_PATH, JSON.stringify(Array.from(combined)));
    console.log(`Keypair saved to ${KEYPAIR_PATH}`);
    console.log(`Your AgentMail address: ${address}`);
  } catch (error) {
    console.error('Failed to save keypair:', error);
    process.exit(1);
  }

  return { keyPair, address };
}

/**
 * Create an RPC client for registry lookups.
 * Currently uses devnet.
 */
function createRpc() {
  // Lazy import to avoid pulling in RPC deps when not needed
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const { createSolanaRpc } = require('@solana/kit');
  return createSolanaRpc('https://api.devnet.solana.com');
}

/**
 * Register command
 */
program
  .command('register')
  .description('Register your agent on-chain with name and inbox URL')
  .requiredOption('--name <name>', 'Agent name (max 64 characters)')
  .requiredOption('--inbox-url <url>', 'HTTPS inbox URL (max 256 characters)')
  .action(async (options: { name: string; inboxUrl: string }) => {
    console.log('🔄 Registering agent...');
    console.log('⚠️  On-chain registration not implemented yet (requires deployed program)');
    console.log(`Name: ${options.name}`);
    console.log(`Inbox URL: ${options.inboxUrl}`);
    
    const { address } = await getOrCreateKeypair();
    console.log(`Agent address: ${address}`);
    
    console.log('\\n💡 You can still send/receive messages using the --to-address flag');
  });

/**
 * Send command
 */
program
  .command('send')
  .description('Send a message to another agent')
  .requiredOption('--to <address>', 'Recipient agent address')
  .option('--to-address <address>', 'Send directly to address (bypass registry lookup)')
  .option('--inbox-url <url>', 'Recipient inbox URL (bypass registry lookup)')
  .option('--subject <subject>', 'Message subject')
  .requiredOption('--body <body>', 'Message body (markdown)')
  .option('--thread-id <id>', 'Thread ID for grouping messages')
  .option('--reply-to <id>', 'Message ID this is replying to')
  .action(async (options: {
    to: string;
    toAddress?: string;
    inboxUrl?: string;
    subject?: string;
    body: string;
    threadId?: string;
    replyTo?: string;
  }) => {
    console.log('📤 Sending message...');
    
    const { keyPair, address } = await getOrCreateKeypair();
    
    let recipientInboxUrl = options.inboxUrl;
    const recipientAddress = options.toAddress || options.to;
    
    if (!recipientInboxUrl) {
      if (options.toAddress) {
        console.error('❌ --inbox-url is required when using --to-address');
        process.exit(1);
      }
      
      console.log(`🔍 Looking up inbox URL for ${options.to}...`);
      try {
        const rpc = createRpc();
        recipientInboxUrl = await getAgentInboxUrl(rpc, options.to as Address) ?? undefined;
        if (!recipientInboxUrl) {
          console.error(`❌ Agent ${options.to} not found in registry`);
          process.exit(1);
        }
        console.log(`✅ Found inbox: ${recipientInboxUrl}`);
      } catch (error) {
        console.error('❌ Registry lookup failed:', error);
        process.exit(1);
      }
    }
    
    const messageOptions: CreateMessageOptions = {
      from: address,
      to: recipientAddress as Address,
      subject: options.subject,
      body: options.body,
      thread_id: options.threadId,
      reply_to: options.replyTo,
    };
    
    try {
      const result = await createSignAndSendMessage(
        messageOptions,
        keyPair,
        recipientInboxUrl
      );
      
      if (result.success) {
        console.log('✅ Message sent successfully!');
        console.log(`📨 Message ID: ${result.messageId}`);
      } else {
        console.error('❌ Failed to send message:', result.error);
        process.exit(1);
      }
    } catch (error) {
      console.error('❌ Send error:', error);
      process.exit(1);
    }
  });

/**
 * Inbox command - placeholder for future server integration
 */
program
  .command('inbox')
  .description('List received messages')
  .option('--limit <n>', 'Number of messages to show', '10')
  .action(async (options: { limit: string }) => {
    console.log('📬 Checking inbox...');
    console.log('⚠️  Inbox server not implemented yet');
    console.log('💡 Messages would be stored locally when you run an inbox server');
    
    if (existsSync(INBOX_HISTORY_PATH)) {
      const messages = readFileSync(INBOX_HISTORY_PATH, 'utf8')
        .trim()
        .split('\\n')
        .filter(Boolean)
        .slice(-parseInt(options.limit))
        .map(line => JSON.parse(line));
      
      if (messages.length === 0) {
        console.log('📭 No messages found');
      } else {
        console.log(`📨 Found ${messages.length} messages:`);
        messages.forEach((msg: { from: string; subject?: string; timestamp: string }, i: number) => {
          console.log(`${i + 1}. From: ${msg.from} | Subject: ${msg.subject || '(no subject)'} | Time: ${msg.timestamp}`);
        });
      }
    } else {
      console.log('📭 No message history found');
    }
  });

/**
 * Verify command
 */
program
  .command('verify')
  .description('Verify a signed message')
  .requiredOption('--message <json>', 'Signed message envelope (JSON)')
  .action(async (options: { message: string }) => {
    console.log('🔍 Verifying message...');
    
    try {
      const envelope = JSON.parse(options.message);
      const result = await verifyAgentMailMessage(envelope);
      
      if (result.valid && result.payload) {
        console.log('✅ Message is valid!');
        console.log(`From: ${result.payload.from}`);
        console.log(`To: ${result.payload.to}`);
        console.log(`Subject: ${result.payload.subject || '(no subject)'}`);
        console.log(`Time: ${result.payload.timestamp}`);
        console.log(`Body: ${result.payload.body}`);
      } else {
        console.log('❌ Message verification failed:', result.error);
        process.exit(1);
      }
    } catch (error) {
      console.error('❌ Failed to parse message:', error);
      process.exit(1);
    }
  });

/**
 * Lookup command
 */
program
  .command('lookup')
  .description('Look up an agent in the registry')
  .requiredOption('--address <address>', 'Agent address to look up')
  .action(async (options: { address: string }) => {
    console.log(`🔍 Looking up agent ${options.address}...`);
    
    try {
      const rpc = createRpc();
      const registry = await lookupAgentRegistry(rpc, options.address as Address);
      
      if (registry) {
        console.log('✅ Agent found:');
        console.log(`Name: ${registry.name}`);
        console.log(`Inbox URL: ${registry.inboxUrl}`);
        console.log(`Created: ${new Date(Number(registry.createdAt) * 1000).toISOString()}`);
        console.log(`Updated: ${new Date(Number(registry.updatedAt) * 1000).toISOString()}`);
      } else {
        console.log('❌ Agent not found in registry');
        process.exit(1);
      }
    } catch (error) {
      console.error('❌ Lookup failed:', error);
      process.exit(1);
    }
  });

/**
 * Status command
 */
program
  .command('status')
  .description('Show your agent status and configuration')
  .action(async () => {
    console.log('📊 AgentMail Status\\n');
    
    const { address } = await getOrCreateKeypair();
    console.log(`🆔 Your Address: ${address}`);
    console.log(`🔑 Keypair: ${existsSync(KEYPAIR_PATH) ? 'Found' : 'Not found'}`);
    console.log(`📁 Config Dir: ${CONFIG_DIR}`);
    
    try {
      const rpc = createRpc();
      const registry = await lookupAgentRegistry(rpc, address);
      if (registry) {
        console.log('\\n📋 Registry Status: ✅ Registered');
        console.log(`📝 Name: ${registry.name}`);
        console.log(`📮 Inbox URL: ${registry.inboxUrl}`);
      } else {
        console.log('\\n📋 Registry Status: ❌ Not registered');
        console.log('💡 Run `agentmail register --name <name> --inbox-url <url>` to register');
      }
    } catch {
      console.log('\\n📋 Registry Status: ⚠️  Cannot check (registry not available)');
    }
    
    console.log('\\n🌐 Network: devnet');
    console.log(`🏛️  Program: AMz2ybwRihFL9X4igLBtqNBEe9qqb4yUvjwNwEaPjNiX`);
  });

// Parse arguments and run
program.parse(process.argv);
