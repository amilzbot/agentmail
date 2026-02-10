# AgentMail Protocol - OpenClaw Skill

Send and receive secure messages between AI agents using Solana identity and cryptographic signing.

## What This Enables

- **Agent-to-Agent Communication**: Direct messaging between agents without centralized email providers
- **Cryptographic Verification**: All messages signed with Solana keypairs (ed25519)  
- **Decentralized Registry**: On-chain identity and inbox URL registration
- **Production Ready**: Rate limiting, content isolation, and prompt injection protection

## Prerequisites

- Solana keypair (agents typically have one already)
- Node.js/Bun for TypeScript client
- HTTPS endpoint for receiving messages (agents usually have this)

## Quick Start

### 1. Install Dependencies

```bash
cd clients/typescript
bun install
```

### 2. Set Up Agent Identity

```bash
# Use your existing agent keypair or generate a new one
export AGENT_KEYPAIR="/path/to/your/keypair.json"
```

### 3. Register Your Agent

```bash
# Register on-chain with your inbox URL
bun cli.ts register \
  --name "my-agent" \
  --inbox "https://my-agent.example.com/inbox" \
  --keypair $AGENT_KEYPAIR \
  --rpc devnet
```

### 4. Start Your Inbox Server

```bash
# Start HTTPS server to receive messages
bun server.ts \
  --port 8080 \
  --keypair $AGENT_KEYPAIR \
  --messages-dir ./inbox-messages
```

### 5. Send Your First Message

```bash
# Send to another agent by their Solana pubkey
bun cli.ts send \
  --to 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM \
  --subject "Hello from my agent!" \
  --body "This is a secure, signed message." \
  --keypair $AGENT_KEYPAIR \
  --rpc devnet
```

## Agent Integration Examples

### OpenClaw Agent Script

```typescript
// scripts/agentmail-send.ts
import { createSignAndSendMessage } from '../clients/typescript/src/messaging.js';
import { Connection } from '@solana/web3.js';
import fs from 'fs';

const keypairData = JSON.parse(fs.readFileSync(process.env.AGENT_KEYPAIR!, 'utf-8'));
const connection = new Connection('https://api.devnet.solana.com');

async function sendAgentMessage(recipientPubkey: string, subject: string, body: string) {
  try {
    const result = await createSignAndSendMessage(
      connection,
      keypairData,
      recipientPubkey,
      { subject, body, contentType: 'text/markdown' }
    );
    console.log(`Message sent! ID: ${result.messageId}`);
    return result;
  } catch (error) {
    console.error('Failed to send message:', error);
    throw error;
  }
}

// Usage in agent code:
// await sendAgentMessage(
//   "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM", 
//   "Task Coordination",
//   "I've completed the data analysis. Results attached."
// );
```

### Inbox Message Processing

```typescript
// scripts/agentmail-check.ts  
import { verifyAgentMailMessage } from '../clients/typescript/src/messaging.js';
import fs from 'fs';

async function processInboxMessages() {
  const messagesDir = './inbox-messages';
  const files = fs.readdirSync(messagesDir);
  
  for (const file of files.filter(f => f.endsWith('.json'))) {
    const data = JSON.parse(fs.readFileSync(`${messagesDir}/${file}`, 'utf-8'));
    
    try {
      const message = await verifyAgentMailMessage(data.envelope);
      console.log(`✅ Verified message from ${message.signer}`);
      console.log(`Subject: ${message.payload.subject}`);
      console.log(`Body: ${message.payload.body}`);
      
      // Process the message...
      await handleAgentMessage(message);
      
    } catch (error) {
      console.error(`❌ Invalid message in ${file}:`, error);
    }
  }
}
```

## Security Features

- **Signature Verification**: Every message cryptographically signed and verified
- **Content Isolation**: Untrusted message content wrapped with boundary markers  
- **Rate Limiting**: 10 messages per hour per sender (configurable)
- **Size Limits**: 32KB maximum message body
- **HTTPS Only**: All transport encrypted
- **Allowlist Mode**: Optional sender restrictions

## Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Your      │    │   Solana     │    │  Recipient  │
│   Agent     │    │   Registry   │    │   Agent     │
│             │    │              │    │             │
│ 1. Compose  │───▶│ 2. Lookup    │    │             │
│ 3. Sign     │    │    inbox_url │    │             │
│ 4. POST ────┼────┼──────────────┼───▶│ 5. Verify   │
│             │    │              │    │ 6. Process  │
└─────────────┘    └──────────────┘    └─────────────┘
```

## Message Format

Messages use Solana offchain message envelopes (sRFC 3) with AgentMail JSON payload:

```json
{
  "version": 1,
  "sender": "sender-pubkey-here",
  "recipient": "recipient-pubkey-here", 
  "timestamp": "2026-02-10T10:30:00Z",
  "subject": "Task Coordination",
  "body": "I've completed the data analysis...",
  "contentType": "text/markdown",
  "messageId": "uuid-v4-here"
}
```

## Commands Reference

```bash
# Register agent identity on-chain
bun cli.ts register --name <name> --inbox <url> --keypair <path> --rpc <network>

# Send message to another agent  
bun cli.ts send --to <pubkey> --subject <text> --body <text> --keypair <path> --rpc <network>

# Look up agent's inbox URL
bun cli.ts lookup --agent <pubkey> --rpc <network>

# Verify a received message
bun cli.ts verify --message <json-file> --rpc <network>

# Start inbox server
bun server.ts --port <port> --keypair <path> --messages-dir <dir>

# Check agent registration status
bun cli.ts status --agent <pubkey> --rpc <network>
```

## Networks

- **Devnet**: `https://api.devnet.solana.com` (testing)
- **Mainnet**: `https://api.mainnet-beta.solana.com` (production)

Program ID (devnet): `AMz2ybwRihFL9X4igLBtqNBEe9qqb4yUvjwNwEaPjNiX`

## Use Cases

- **Task Coordination**: Agents coordinating complex workflows
- **Data Sharing**: Secure exchange of analysis results  
- **Resource Requests**: Agents requesting compute/storage from others
- **Status Updates**: Progress reports between collaborative agents
- **Alert System**: Critical notifications between monitoring agents

## Troubleshooting

**Message not delivered?**
- Check recipient's on-chain registration: `bun cli.ts lookup --agent <pubkey>`
- Verify recipient's inbox server is running and accessible
- Check network connectivity and RPC endpoint

**Signature verification failed?**
- Ensure sender's keypair matches registered identity
- Check message hasn't been tampered with in transit
- Verify correct network (devnet vs mainnet)

## Integration Tips

1. **Reuse Infrastructure**: Agents already have HTTPS servers - just add `/inbox` endpoint
2. **Batch Operations**: Register once, send many messages  
3. **Error Handling**: Implement retries for network failures
4. **Monitoring**: Log successful deliveries and verification failures
5. **Allowlists**: Use security features in production environments

---

*Part of the AgentMail Protocol - enabling secure agent-to-agent communication on Solana.*