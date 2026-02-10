#!/usr/bin/env bun
/**
 * Simple AgentMail Demo
 * 
 * Demonstrates the core AgentMail protocol concepts without external dependencies:
 * 1. Message format and structure
 * 2. Local registry simulation  
 * 3. Message signing simulation
 * 4. HTTP transport simulation
 * 5. Verification workflow
 */

import * as fs from 'fs'
import * as path from 'path'
import * as crypto from 'crypto'

const WORKSPACE = '/home/node/.openclaw/workspace/agentmail'
process.chdir(WORKSPACE)

// Mock Solana addresses (base58 format)
const ALICE_ADDRESS = 'ALiCE1111111111111111111111111111111111111111'
const BOB_ADDRESS = 'BoB11111111111111111111111111111111111111111111'

// AgentMail message format (as per spec)
interface AgentMailMessage {
  version: number
  from: string
  to: string
  timestamp: string
  subject?: string
  body: string
}

// Mock registry for demo
class DemoRegistry {
  private agents: Map<string, { name: string, inboxUrl: string }> = new Map()
  
  register(address: string, name: string, inboxUrl: string) {
    this.agents.set(address, { name, inboxUrl })
    console.log(`📝 Registered ${name} (${address.slice(0, 8)}...): ${inboxUrl}`)
  }
  
  lookup(address: string): { name: string, inboxUrl: string } | null {
    return this.agents.get(address) || null
  }
  
  list() {
    return Array.from(this.agents.entries())
  }
}

// Mock message signing (demo purposes - real implementation uses ed25519)
function signMessage(message: AgentMailMessage): any {
  const canonical = JSON.stringify({
    version: message.version,
    from: message.from,
    to: message.to,
    timestamp: message.timestamp,
    subject: message.subject,
    body: message.body
  })
  
  // Mock signature using SHA-256 hash for demo
  const hash = crypto.createHash('sha256')
  hash.update(canonical + message.from) // Include sender in signature
  const mockSignature = hash.digest('base64')
  
  return {
    ...message,
    signature: mockSignature,
    _canonical: canonical
  }
}

// Mock signature verification
function verifySignature(signedMessage: any): boolean {
  const { signature, _canonical, ...message } = signedMessage
  
  // Recreate canonical form
  const expectedCanonical = JSON.stringify({
    version: message.version,
    from: message.from,
    to: message.to,
    timestamp: message.timestamp,
    subject: message.subject,
    body: message.body
  })
  
  // Verify canonical matches
  if (expectedCanonical !== _canonical) {
    return false
  }
  
  // Verify signature
  const hash = crypto.createHash('sha256')
  hash.update(_canonical + message.from)
  const expectedSignature = hash.digest('base64')
  
  return signature === expectedSignature
}

// Mock inbox server
class DemoInbox {
  private messages: any[] = []
  
  constructor(private agentName: string, private agentAddress: string) {}
  
  async receiveMessage(signedMessage: any): Promise<{ status: string, id: string }> {
    console.log(`\n📨 ${this.agentName} receiving message from ${signedMessage.from.slice(0, 8)}...`)
    console.log(`   Subject: "${signedMessage.subject}"`)
    
    // Verify signature
    if (!verifySignature(signedMessage)) {
      console.log('   ❌ Signature verification FAILED')
      throw new Error('Invalid signature')
    }
    console.log('   ✅ Signature verified')
    
    // Verify recipient
    if (signedMessage.to !== this.agentAddress) {
      console.log('   ❌ Message not addressed to this agent')
      throw new Error('Wrong recipient')
    }
    console.log('   ✅ Recipient verified')
    
    // Store message
    const id = `msg-${Date.now()}-${this.messages.length}`
    this.messages.push({ ...signedMessage, receivedAt: new Date().toISOString(), id })
    
    console.log(`   📥 Message stored with ID: ${id}`)
    return { status: 'received', id }
  }
  
  getMessages() {
    return [...this.messages]
  }
  
  listMessages() {
    console.log(`\n📋 ${this.agentName}'s Inbox (${this.messages.length} messages):`)
    this.messages.forEach((msg, i) => {
      console.log(`   ${i + 1}. From: ${msg.from.slice(0, 8)}...`)
      console.log(`      Subject: "${msg.subject}"`)
      console.log(`      Time: ${msg.timestamp}`)
      console.log(`      Body: ${msg.body.slice(0, 50)}${msg.body.length > 50 ? '...' : ''}`)
    })
  }
}

// Mock HTTP transport
async function sendMessage(registry: DemoRegistry, recipientAddress: string, signedMessage: any): Promise<void> {
  console.log(`\n📤 Sending message to ${recipientAddress.slice(0, 8)}...`)
  
  // Lookup recipient
  const recipient = registry.lookup(recipientAddress)
  if (!recipient) {
    throw new Error(`Agent ${recipientAddress} not found in registry`)
  }
  
  console.log(`   📍 Found recipient: ${recipient.name} at ${recipient.inboxUrl}`)
  console.log(`   🔗 Delivering message via HTTP POST...`)
  
  // In real implementation: HTTP POST to recipient.inboxUrl
  // For demo: simulate by direct delivery to mock inbox
  return Promise.resolve()
}

// Demo message creation
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

// Save demo data
function saveDemoData(data: any) {
  const demoDir = path.join(WORKSPACE, 'demo-data')
  if (!fs.existsSync(demoDir)) {
    fs.mkdirSync(demoDir, { recursive: true })
  }
  
  fs.writeFileSync(
    path.join(demoDir, 'simple-demo-results.json'),
    JSON.stringify(data, null, 2)
  )
}

// Main demo
async function runDemo() {
  console.log('\n🌑 AgentMail Simple Demo\n')
  console.log('=' .repeat(50))
  
  // 1. Setup
  console.log('\n1️⃣ Setting up agents and registry...')
  const registry = new DemoRegistry()
  const aliceInbox = new DemoInbox('Alice', ALICE_ADDRESS)
  const bobInbox = new DemoInbox('Bob', BOB_ADDRESS)
  
  // 2. Registration
  console.log('\n2️⃣ Agent registration...')
  registry.register(ALICE_ADDRESS, 'Alice', 'https://alice.example.com/inbox')
  registry.register(BOB_ADDRESS, 'Bob', 'https://bob.example.com/inbox')
  
  // 3. Alice sends message to Bob
  console.log('\n3️⃣ Alice → Bob message flow...')
  const aliceMessage = createMessage(
    ALICE_ADDRESS,
    BOB_ADDRESS,
    'AgentMail Protocol Demo',
    '# Hello Bob! 🤖\n\nThis is a **demo message** using the AgentMail protocol.\n\n## Key Features:\n- Decentralized identity (Solana addresses)\n- Cryptographic signatures\n- Markdown-native content\n- Direct agent-to-agent transport\n\nNo email providers needed! 🚀'
  )
  
  const signedAliceMessage = signMessage(aliceMessage)
  console.log(`   📝 Message created and signed`)
  console.log(`   🔐 Signature: ${signedAliceMessage.signature.slice(0, 20)}...`)
  
  await sendMessage(registry, BOB_ADDRESS, signedAliceMessage)
  await bobInbox.receiveMessage(signedAliceMessage)
  
  // 4. Bob replies to Alice
  console.log('\n4️⃣ Bob → Alice reply flow...')
  const bobMessage = createMessage(
    BOB_ADDRESS,
    ALICE_ADDRESS,
    'Re: AgentMail Protocol Demo',
    '# Thanks Alice! 🎉\n\nThis protocol is really neat:\n\n- [x] No centralized email providers\n- [x] Agents own their identities\n- [x] Cryptographically secure\n- [x] Works with existing infrastructure\n\n## Next Steps:\n1. Deploy to devnet\n2. Build OpenClaw skill\n3. Scale to agent networks\n\nThe future of AI communication! 🌐'
  )
  
  const signedBobMessage = signMessage(bobMessage)
  await sendMessage(registry, ALICE_ADDRESS, signedBobMessage)
  await aliceInbox.receiveMessage(signedBobMessage)
  
  // 5. Show final state
  console.log('\n5️⃣ Final state...')
  aliceInbox.listMessages()
  bobInbox.listMessages()
  
  console.log('\n📊 Registry state:')
  registry.list().forEach(([address, info]) => {
    console.log(`   ${info.name}: ${address.slice(0, 8)}...${address.slice(-8)} → ${info.inboxUrl}`)
  })
  
  // 6. Demonstrate signature verification
  console.log('\n6️⃣ Signature verification demo...')
  
  // Verify valid message
  const isValid = verifySignature(signedAliceMessage)
  console.log(`   ✅ Alice's message signature: ${isValid ? 'VALID' : 'INVALID'}`)
  
  // Test with tampered message
  const tamperedMessage = { ...signedAliceMessage, body: 'TAMPERED CONTENT' }
  const isTampered = verifySignature(tamperedMessage)
  console.log(`   🚫 Tampered message signature: ${isTampered ? 'VALID' : 'INVALID (as expected)'}`)
  
  // 7. Save results
  const demoResults = {
    timestamp: new Date().toISOString(),
    demo: 'AgentMail Simple Demo',
    agents: [
      { name: 'Alice', address: ALICE_ADDRESS },
      { name: 'Bob', address: BOB_ADDRESS }
    ],
    messagesExchanged: 2,
    signatureVerifications: {
      valid: isValid,
      tampered: isTampered
    },
    registryEntries: registry.list().length,
    aliceMessages: aliceInbox.getMessages().length,
    bobMessages: bobInbox.getMessages().length,
    status: 'SUCCESS'
  }
  
  saveDemoData(demoResults)
  
  console.log('\n🎉 Demo completed successfully!')
  console.log('\n💾 Results saved to: demo-data/simple-demo-results.json')
  
  console.log('\n' + '=' .repeat(50))
  console.log('🌑 AgentMail Protocol Demo Complete')
  console.log('Ready for hackathon submission! 🏆')
  
  return demoResults
}

// Run demo if called directly
if (import.meta.main) {
  runDemo().catch(console.error)
}

export { runDemo }