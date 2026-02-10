#!/usr/bin/env bun
/**
 * OpenClaw AgentMail Integration Helper
 * 
 * Provides simple functions for agents to send/receive messages
 * via the AgentMail protocol.
 */

import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { createSignAndSendMessage, verifyAgentMailMessage } from '../clients/typescript/src/messaging.js';
import { lookupAgentRegistry } from '../clients/typescript/src/registry.js';
import fs from 'fs';
import path from 'path';

export interface AgentMailConfig {
  keypairPath: string;
  rpcUrl: string;
  messagesDir?: string;
}

export class AgentMailClient {
  private connection: Connection;
  private keypair: Keypair;
  private messagesDir: string;

  constructor(config: AgentMailConfig) {
    this.connection = new Connection(config.rpcUrl);
    
    const keypairData = JSON.parse(fs.readFileSync(config.keypairPath, 'utf-8'));
    this.keypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
    
    this.messagesDir = config.messagesDir || './agentmail-messages';
    if (!fs.existsSync(this.messagesDir)) {
      fs.mkdirSync(this.messagesDir, { recursive: true });
    }
  }

  /**
   * Send a message to another agent
   */
  async sendMessage(
    recipientPubkey: string,
    subject: string, 
    body: string,
    contentType: string = 'text/markdown'
  ) {
    try {
      const result = await createSignAndSendMessage(
        this.connection,
        Array.from(this.keypair.secretKey),
        recipientPubkey,
        { subject, body, contentType }
      );
      
      console.log(`📨 Message sent to ${recipientPubkey.slice(0, 8)}... | ID: ${result.messageId}`);
      return result;
    } catch (error) {
      console.error('❌ Failed to send message:', error);
      throw error;
    }
  }

  /**
   * Check for new messages in inbox
   */
  async checkMessages(): Promise<any[]> {
    if (!fs.existsSync(this.messagesDir)) {
      return [];
    }

    const messages = [];
    const files = fs.readdirSync(this.messagesDir)
      .filter(f => f.endsWith('.json'))
      .sort(); // Process in chronological order

    for (const file of files) {
      try {
        const filePath = path.join(this.messagesDir, file);
        const data = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
        
        const verified = await verifyAgentMailMessage(data.envelope);
        messages.push({
          file,
          verified,
          receivedAt: data.receivedAt,
          senderIp: data.senderIp
        });
      } catch (error) {
        console.error(`❌ Failed to verify message in ${file}:`, error.message);
      }
    }

    return messages;
  }

  /**
   * Look up an agent's inbox URL
   */
  async lookupAgent(pubkey: string): Promise<string | null> {
    try {
      const registry = await lookupAgentRegistry(this.connection, new PublicKey(pubkey));
      return registry?.inbox_url || null;
    } catch (error) {
      console.error(`❌ Failed to lookup agent ${pubkey}:`, error);
      return null;
    }
  }

  /**
   * Mark a message as processed (move to processed folder)
   */
  markAsProcessed(messageFile: string): void {
    const processedDir = path.join(this.messagesDir, 'processed');
    if (!fs.existsSync(processedDir)) {
      fs.mkdirSync(processedDir, { recursive: true });
    }
    
    const sourcePath = path.join(this.messagesDir, messageFile);
    const destPath = path.join(processedDir, messageFile);
    
    fs.renameSync(sourcePath, destPath);
    console.log(`📁 Message ${messageFile} marked as processed`);
  }

  /**
   * Get agent's own public key
   */
  getPublicKey(): string {
    return this.keypair.publicKey.toBase58();
  }
}

// Example usage for OpenClaw agents
if (import.meta.main) {
  const config: AgentMailConfig = {
    keypairPath: process.env.AGENT_KEYPAIR || '/home/node/.openclaw/workspace/.keys/nix.json',
    rpcUrl: process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com',
    messagesDir: process.env.AGENTMAIL_MESSAGES_DIR || './agentmail-messages'
  };

  const client = new AgentMailClient(config);
  
  // Demo: send a message
  if (process.argv[2] === 'send') {
    const recipient = process.argv[3];
    const subject = process.argv[4] || 'Test message';
    const body = process.argv[5] || 'Hello from OpenClaw agent!';
    
    await client.sendMessage(recipient, subject, body);
  }
  
  // Demo: check messages
  if (process.argv[2] === 'check') {
    const messages = await client.checkMessages();
    console.log(`📬 Found ${messages.length} messages`);
    
    for (const msg of messages) {
      console.log(`\n📨 Message from: ${msg.verified.signer.slice(0, 8)}...`);
      console.log(`   Subject: ${msg.verified.payload.subject}`);
      console.log(`   Body: ${msg.verified.payload.body.slice(0, 100)}...`);
      console.log(`   Received: ${msg.receivedAt}`);
      
      // Auto-mark as processed for demo
      client.markAsProcessed(msg.file);
    }
  }

  // Demo: lookup agent
  if (process.argv[2] === 'lookup') {
    const pubkey = process.argv[3];
    const inboxUrl = await client.lookupAgent(pubkey);
    console.log(`📍 ${pubkey} -> ${inboxUrl || 'Not registered'}`);
  }
}

export { AgentMailConfig, AgentMailClient };