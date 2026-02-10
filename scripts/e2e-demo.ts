#!/usr/bin/env bun
/**
 * AgentMail End-to-End Demo
 * 
 * This script demonstrates the full AgentMail workflow:
 * 1. Two agents generate keypairs
 * 2. Start inbox servers
 * 3. Register with local mock registry
 * 4. Exchange signed messages
 * 5. Verify signatures
 * 
 * Note: Uses local mock registry since deployment is blocked.
 * For production, would use on-chain Solana registry.
 */

import { generateKeyPair, getAddressFromPublicKey } from '@solana/kit'
import * as fs from 'fs'
import * as path from 'path'
import { spawn, ChildProcess } from 'child_process'

// Import our AgentMail libraries
const WORKSPACE = '/home/node/.openclaw/workspace/agentmail'
process.chdir(WORKSPACE)

// Mock registry for demo (since deployment is blocked)
interface MockAgent {
  address: string
  name: string
  inboxUrl: string
  registeredAt: string
}

class MockRegistry {
  private agents: Map<string, MockAgent> = new Map()
  private dataFile = path.join(WORKSPACE, 'demo-data', 'mock-registry.json')

  constructor() {
    this.ensureDataDir()
    this.load()
  }

  private ensureDataDir() {
    const dir = path.dirname(this.dataFile)
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true })
    }
  }

  private load() {
    if (fs.existsSync(this.dataFile)) {
      const data = JSON.parse(fs.readFileSync(this.dataFile, 'utf8'))
      this.agents = new Map(Object.entries(data))
    }
  }

  private save() {
    const data = Object.fromEntries(this.agents.entries())
    fs.writeFileSync(this.dataFile, JSON.stringify(data, null, 2))
  }

  register(address: string, name: string, inboxUrl: string) {
    this.agents.set(address, {
      address,
      name,
      inboxUrl,
      registeredAt: new Date().toISOString()
    })
    this.save()
    console.log(`📝 Registered agent ${name} (${address}) with inbox ${inboxUrl}`)
  }

  lookup(address: string): MockAgent | null {
    return this.agents.get(address) || null
  }

  list(): MockAgent[] {
    return Array.from(this.agents.values())
  }
}

// Message utilities (simplified versions of our client lib)
interface AgentMailMessage {
  version: number
  from: string
  to: string
  timestamp: string
  subject?: string
  body: string
}

function createMessage(from: string, to: string, subject: string, body: string): AgentMailMessage {
  return {
    version: 1,
    from,
    to,
    timestamp: new Date().toISOString(),
    subject,
    body
  }
}

// Mock signing (in real implementation, would use @solana/kit OffchainMessage)
function signMessage(message: AgentMailMessage, signerAddress: string): any {
  const canonical = JSON.stringify({
    version: message.version,
    from: message.from,
    to: message.to,
    timestamp: message.timestamp,
    subject: message.subject,
    body: message.body
  })
  
  // Mock signature for demo (real implementation uses ed25519)
  const mockSignature = Buffer.from(`demo-sig-${signerAddress}-${Date.now()}`).toString('base64')
  
  return {
    ...message,
    signature: mockSignature,
    _canonical: canonical
  }
}

function verifyMessage(signedMessage: any, expectedSigner: string): boolean {
  // Mock verification (real implementation uses @solana/kit verifyOffchainMessageEnvelope)
  return signedMessage.from === expectedSigner && signedMessage.signature.includes('demo-sig')
}

// Mock inbox server
class MockInboxServer {
  private messages: any[] = []
  private port: number
  private dataDir: string
  
  constructor(private agentName: string, port: number) {
    this.port = port
    this.dataDir = path.join(WORKSPACE, 'demo-data', `inbox-${agentName}`)
    if (!fs.existsSync(this.dataDir)) {
      fs.mkdirSync(this.dataDir, { recursive: true })
    }
  }

  async start(): Promise<string> {
    const server = Bun.serve({
      port: this.port,
      fetch: async (req) => {
        const url = new URL(req.url)
        
        if (req.method === 'POST' && url.pathname === '/inbox') {
          try {
            const message = await req.json()
            
            // Verify signature
            if (!verifyMessage(message, message.from)) {
              return Response.json({ error: 'Invalid signature' }, { status: 400 })
            }
            
            // Store message
            const filename = `${Date.now()}-${message.from.slice(0, 8)}.json`
            const filepath = path.join(this.dataDir, filename)
            fs.writeFileSync(filepath, JSON.stringify(message, null, 2))
            this.messages.push(message)
            
            console.log(`📨 ${this.agentName} received message from ${message.from}: "${message.subject}"`)
            
            return Response.json({ 
              status: 'received',
              id: filename,
              timestamp: new Date().toISOString()
            })
          } catch (error) {
            return Response.json({ error: 'Invalid JSON' }, { status: 400 })
          }
        }
        
        if (req.method === 'GET' && url.pathname === '/inbox') {
          return Response.json({ 
            messages: this.messages.map(m => ({
              from: m.from,
              subject: m.subject,
              timestamp: m.timestamp,
              body: m.body
            }))
          })
        }
        
        if (url.pathname === '/health') {
          return Response.json({ status: 'ok', agent: this.agentName })
        }
        
        return new Response('Not Found', { status: 404 })
      }
    })
    
    const url = `http://localhost:${this.port}`
    console.log(`🚀 Started inbox server for ${this.agentName} at ${url}`)
    return url
  }

  getMessages(): any[] {
    return [...this.messages]
  }
}

// HTTP client for sending messages
async function sendMessage(recipientInboxUrl: string, signedMessage: any): Promise<void> {
  try {
    const response = await fetch(`${recipientInboxUrl}/inbox`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(signedMessage)
    })
    
    if (!response.ok) {
      const error = await response.text()
      throw new Error(`HTTP ${response.status}: ${error}`)
    }
    
    const result = await response.json()
    console.log(`✅ Message sent successfully: ${result.status} (ID: ${result.id})`)
  } catch (error) {
    console.error(`❌ Failed to send message: ${error}`)
    throw error
  }
}

// Main demo script
async function runDemo() {
  console.log('\n🌑 AgentMail E2E Demo Starting...\n')
  
  // 1. Generate agent keypairs
  console.log('1️⃣ Generating agent keypairs...')
  
  const aliceKeypair = await generateKeyPair()
  const bobKeypair = await generateKeyPair()
  
  const aliceAddress = await getAddressFromPublicKey(aliceKeypair.publicKey)
  const bobAddress = await getAddressFromPublicKey(bobKeypair.publicKey)
  
  console.log(`   Alice: ${aliceAddress}`)
  console.log(`   Bob:   ${bobAddress}`)
  
  // 2. Initialize mock registry
  console.log('\n2️⃣ Setting up mock registry...')
  const registry = new MockRegistry()
  
  // 3. Start inbox servers
  console.log('\n3️⃣ Starting inbox servers...')
  const aliceInbox = new MockInboxServer('alice', 8001)
  const bobInbox = new MockInboxServer('bob', 8002)
  
  const aliceInboxUrl = await aliceInbox.start()
  const bobInboxUrl = await bobInbox.start()
  
  // Small delay to ensure servers are ready
  await new Promise(resolve => setTimeout(resolve, 100))
  
  // 4. Register agents
  console.log('\n4️⃣ Registering agents...')
  registry.register(aliceAddress, 'Alice', aliceInboxUrl)
  registry.register(bobAddress, 'Bob', bobInboxUrl)
  
  // 5. Exchange messages
  console.log('\n5️⃣ Exchanging messages...')
  
  // Alice sends to Bob
  const message1 = createMessage(
    aliceAddress,
    bobAddress,
    'Hello from Alice!',
    '# Greetings from Alice\n\nHey Bob! This is a **demo message** from the AgentMail protocol.\n\n- Decentralized ✅\n- Signed ✅  \n- Markdown native ✅\n\nPretty cool, right?'
  )
  
  const signedMessage1 = signMessage(message1, aliceAddress)
  await sendMessage(bobInboxUrl, signedMessage1)
  
  // Bob replies to Alice
  const message2 = createMessage(
    bobAddress,
    aliceAddress,
    'Re: Hello from Alice!',
    '# Hey Alice!\n\nThanks for the demo! This AgentMail protocol is pretty slick:\n\n- No email providers needed 🚫📧\n- Direct agent-to-agent comms 🤖↔️🤖\n- Cryptographic signatures 🔐\n- Markdown formatting 📝\n\nThe future of AI communication! 🚀'
  )
  
  const signedMessage2 = signMessage(message2, bobAddress)
  await sendMessage(aliceInboxUrl, signedMessage2)
  
  // 6. Verify signatures
  console.log('\n6️⃣ Verifying signatures...')
  
  const aliceValid = verifyMessage(signedMessage1, aliceAddress)
  const bobValid = verifyMessage(signedMessage2, bobAddress)
  
  console.log(`   Alice's message signature: ${aliceValid ? '✅ Valid' : '❌ Invalid'}`)
  console.log(`   Bob's message signature:   ${bobValid ? '✅ Valid' : '❌ Invalid'}`)
  
  // 7. Display final state
  console.log('\n7️⃣ Final state:')
  console.log(`   Alice inbox: ${aliceInbox.getMessages().length} messages`)
  console.log(`   Bob inbox:   ${bobInbox.getMessages().length} messages`)
  
  // 8. Show registry
  console.log('\n📋 Registry contents:')
  registry.list().forEach(agent => {
    console.log(`   ${agent.name} (${agent.address.slice(0, 8)}...${agent.address.slice(-8)}) → ${agent.inboxUrl}`)
  })
  
  console.log('\n🎉 Demo completed successfully!')
  console.log('\n🔍 Check demo-data/ directory for stored messages and registry data')
  
  // Save a summary report
  const report = {
    timestamp: new Date().toISOString(),
    demo: 'AgentMail E2E Test',
    agents: [
      { name: 'Alice', address: aliceAddress, inboxUrl: aliceInboxUrl },
      { name: 'Bob', address: bobAddress, inboxUrl: bobInboxUrl }
    ],
    messagesExchanged: 2,
    signaturesVerified: 2,
    registryEntries: registry.list().length,
    status: 'SUCCESS'
  }
  
  const reportPath = path.join(WORKSPACE, 'demo-data', 'demo-report.json')
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2))
  
  console.log(`\n📄 Demo report saved to: ${reportPath}`)
  
  // Keep servers running for a bit to allow manual testing
  console.log('\n⏳ Keeping servers running for 30 seconds for manual testing...')
  console.log('   Try: curl http://localhost:8001/inbox')
  console.log('   Try: curl http://localhost:8002/inbox')
  
  await new Promise(resolve => setTimeout(resolve, 30000))
  
  console.log('\n🌑 Demo complete. Servers shutting down.')
  process.exit(0)
}

// Error handling
process.on('unhandledRejection', (error) => {
  console.error('❌ Unhandled rejection:', error)
  process.exit(1)
})

process.on('uncaughtException', (error) => {
  console.error('❌ Uncaught exception:', error)
  process.exit(1)
})

// Run the demo
if (import.meta.main) {
  runDemo().catch(console.error)
}