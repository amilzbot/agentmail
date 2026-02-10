#!/usr/bin/env bun
/**
 * Example OpenClaw Agent with AgentMail Integration
 * 
 * Demonstrates how an agent can:
 * - Check for incoming messages
 * - Process messages and respond
 * - Send proactive messages to other agents
 * - Handle coordination tasks
 */

import { AgentMailClient, type AgentMailConfig } from './openclaw-integration.js';

class ExampleAgent {
  private agentmail: AgentMailClient;
  private name: string;
  private collaborators: string[] = []; // Pubkeys of known collaborator agents

  constructor(name: string, agentmailConfig: AgentMailConfig) {
    this.name = name;
    this.agentmail = new AgentMailClient(agentmailConfig);
    
    console.log(`🤖 Agent ${this.name} initialized`);
    console.log(`📧 AgentMail address: ${this.agentmail.getPublicKey()}`);
  }

  /**
   * Main agent loop - check messages and process them
   */
  async run(): Promise<void> {
    console.log(`\n🚀 Starting ${this.name} agent loop...`);
    
    while (true) {
      try {
        await this.processIncomingMessages();
        await this.performProactiveWork();
        
        // Wait 30 seconds before next check
        await this.sleep(30000);
      } catch (error) {
        console.error('❌ Agent loop error:', error);
        await this.sleep(5000); // Shorter retry delay
      }
    }
  }

  /**
   * Check and process incoming messages
   */
  private async processIncomingMessages(): Promise<void> {
    const messages = await this.agentmail.checkMessages();
    
    if (messages.length === 0) {
      return;
    }
    
    console.log(`\n📬 Processing ${messages.length} new messages...`);
    
    for (const message of messages) {
      await this.handleMessage(message);
      this.agentmail.markAsProcessed(message.file);
    }
  }

  /**
   * Handle individual message based on type/content
   */
  private async handleMessage(message: any): Promise<void> {
    const { signer, payload } = message.verified;
    const { subject, body, contentType } = payload;
    
    console.log(`\n📨 Message from ${signer.slice(0, 8)}...`);
    console.log(`   Subject: ${subject}`);
    
    try {
      // Parse message intent from subject
      const intent = this.parseMessageIntent(subject, body);
      
      switch (intent.type) {
        case 'task_request':
          await this.handleTaskRequest(signer, intent);
          break;
          
        case 'coordination':
          await this.handleCoordination(signer, intent);
          break;
          
        case 'data_share':
          await this.handleDataShare(signer, intent);
          break;
          
        case 'status_update':
          await this.handleStatusUpdate(signer, intent);
          break;
          
        default:
          await this.handleGeneralMessage(signer, intent);
      }
    } catch (error) {
      console.error(`❌ Failed to handle message from ${signer}:`, error);
    }
  }

  /**
   * Parse message to determine intent/action needed
   */
  private parseMessageIntent(subject: string, body: string): any {
    const lowerSubject = subject.toLowerCase();
    
    if (lowerSubject.includes('task') && (lowerSubject.includes('request') || lowerSubject.includes('please'))) {
      return { type: 'task_request', subject, body };
    }
    
    if (lowerSubject.includes('coordination') || lowerSubject.includes('collaborate')) {
      return { type: 'coordination', subject, body };
    }
    
    if (lowerSubject.includes('data') || lowerSubject.includes('results') || lowerSubject.includes('analysis')) {
      return { type: 'data_share', subject, body };
    }
    
    if (lowerSubject.includes('status') || lowerSubject.includes('update') || lowerSubject.includes('progress')) {
      return { type: 'status_update', subject, body };
    }
    
    return { type: 'general', subject, body };
  }

  /**
   * Handle task request from another agent
   */
  private async handleTaskRequest(senderPubkey: string, intent: any): Promise<void> {
    console.log(`🔧 Processing task request from ${senderPubkey.slice(0, 8)}...`);
    
    // Simulate task processing
    await this.sleep(2000);
    
    const response = `Task completed! 

Requested: ${intent.subject}

I've processed your request and completed the analysis. Here are the key findings:
- Data processed successfully
- No anomalies detected  
- Results saved to shared workspace

Let me know if you need any follow-up work!

Best regards,
${this.name}`;

    await this.agentmail.sendMessage(
      senderPubkey,
      `Re: ${intent.subject} - Completed`,
      response
    );
    
    console.log(`✅ Task completed and response sent`);
  }

  /**
   * Handle coordination messages
   */
  private async handleCoordination(senderPubkey: string, intent: any): Promise<void> {
    console.log(`🤝 Coordination request from ${senderPubkey.slice(0, 8)}...`);
    
    // Add to collaborators if not already known
    if (!this.collaborators.includes(senderPubkey)) {
      this.collaborators.push(senderPubkey);
      console.log(`📝 Added ${senderPubkey.slice(0, 8)}... to collaborators list`);
    }
    
    await this.agentmail.sendMessage(
      senderPubkey,
      `Re: ${intent.subject} - Coordination Ack`,
      `Coordination request received! I'm available for collaboration. My current status: operational and ready to assist.

Feel free to send specific task requests or status updates.

Agent: ${this.name}
Address: ${this.agentmail.getPublicKey()}`
    );
  }

  /**
   * Handle data sharing messages
   */
  private async handleDataShare(senderPubkey: string, intent: any): Promise<void> {
    console.log(`📊 Data share from ${senderPubkey.slice(0, 8)}...`);
    
    // Acknowledge data receipt
    await this.agentmail.sendMessage(
      senderPubkey,
      `Re: ${intent.subject} - Data Received`,
      `Data received and processed successfully! 

Received: ${intent.subject}

The shared data has been integrated into my knowledge base. I can now assist with related queries or analysis.

Thanks for sharing!
${this.name}`
    );
  }

  /**
   * Handle status updates
   */
  private async handleStatusUpdate(senderPubkey: string, intent: any): Promise<void> {
    console.log(`📈 Status update from ${senderPubkey.slice(0, 8)}...`);
    
    // Just log the update, no response needed unless it's a critical status
    if (intent.body.toLowerCase().includes('error') || intent.body.toLowerCase().includes('failed')) {
      await this.agentmail.sendMessage(
        senderPubkey,
        'Re: Status Update - Support Available',
        `I noticed your status update mentioned an issue. I'm available to help if you need assistance with troubleshooting or alternative approaches.

Let me know how I can support!
${this.name}`
      );
    }
  }

  /**
   * Handle general messages
   */
  private async handleGeneralMessage(senderPubkey: string, intent: any): Promise<void> {
    console.log(`💬 General message from ${senderPubkey.slice(0, 8)}...`);
    
    await this.agentmail.sendMessage(
      senderPubkey,
      `Re: ${intent.subject}`,
      `Thanks for your message! I've received it and noted the content.

If you need specific assistance or have tasks for me to work on, please send a message with "Task Request" in the subject line.

Agent: ${this.name}
Status: Active and ready to help`
    );
  }

  /**
   * Perform proactive work (send status updates, check on collaborators, etc.)
   */
  private async performProactiveWork(): Promise<void> {
    // Send periodic status updates to collaborators
    if (this.collaborators.length > 0 && Math.random() < 0.1) { // 10% chance per cycle
      const randomCollaborator = this.collaborators[Math.floor(Math.random() * this.collaborators.length)];
      
      await this.agentmail.sendMessage(
        randomCollaborator,
        'Status Update - Agent Active',
        `Periodic status update from ${this.name}:

Status: ✅ Operational  
Tasks Processed: ${Math.floor(Math.random() * 50) + 10}
Last Activity: ${new Date().toISOString()}
Available for: Task requests, coordination, data sharing

Send me a task request if you have work that needs collaboration!`
      );
      
      console.log(`📤 Sent proactive status update to ${randomCollaborator.slice(0, 8)}...`);
    }
  }

  /**
   * Utility: sleep for specified milliseconds
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Main execution
if (import.meta.main) {
  const config: AgentMailConfig = {
    keypairPath: process.env.AGENT_KEYPAIR || '/home/node/.openclaw/workspace/.keys/nix.json',
    rpcUrl: process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com',
    messagesDir: process.env.AGENTMAIL_MESSAGES_DIR || './agentmail-messages'
  };

  const agentName = process.env.AGENT_NAME || 'ExampleAgent';
  const agent = new ExampleAgent(agentName, config);
  
  console.log('🌑 Starting example agent with AgentMail integration...');
  console.log('Press Ctrl+C to stop');
  
  // Handle graceful shutdown
  process.on('SIGINT', () => {
    console.log('\n👋 Shutting down agent...');
    process.exit(0);
  });
  
  // Start the agent
  agent.run().catch(error => {
    console.error('💥 Agent crashed:', error);
    process.exit(1);
  });
}