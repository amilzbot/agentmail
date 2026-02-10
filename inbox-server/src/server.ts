#!/usr/bin/env bun

/**
 * AgentMail Inbox Server
 * 
 * HTTP server that receives signed AgentMail messages via POST /inbox
 * and stores them for retrieval via GET /inbox.
 */

import { verifyOffchainMessageEnvelope } from '@solana/kit'
import { mkdir, writeFile, readdir, readFile } from 'fs/promises'
import { join } from 'path'
import { existsSync } from 'fs'
import { SecurityManager, wrapUntrustedContent } from './security.js'

interface AgentMailMessage {
  version: number
  from: string
  to: string
  timestamp: string
  subject?: string
  body: string
}

interface StoredMessage {
  id: string
  receivedAt: string
  envelope: any
  message: AgentMailMessage
  sender: string
}

class InboxServer {
  private readonly port: number
  private readonly dataDir: string
  private readonly security: SecurityManager
  
  constructor(port = 3000, dataDir = './inbox-data', securityConfig = {}) {
    this.port = port
    this.dataDir = dataDir
    this.security = new SecurityManager(securityConfig)
    
    // Set up periodic cleanup
    setInterval(() => this.security.cleanup(), 10 * 60 * 1000)
  }

  async init() {
    // Ensure data directory exists
    if (!existsSync(this.dataDir)) {
      await mkdir(this.dataDir, { recursive: true })
    }
    console.log(`📦 Data directory: ${this.dataDir}`)
  }

  async handleInboxPost(request: Request): Promise<Response> {
    try {
      // Check content length before parsing
      const contentLength = request.headers.get('content-length')
      if (contentLength && parseInt(contentLength) > this.security.getConfig().maxBodySize) {
        return new Response(
          JSON.stringify({ error: 'Request body too large' }),
          { status: 413, headers: { 'Content-Type': 'application/json' } }
        )
      }

      const envelope = await request.json()
      
      // Verify the offchain message signature
      const isValid = await verifyOffchainMessageEnvelope(envelope)
      if (!isValid) {
        return new Response(
          JSON.stringify({ error: 'Invalid message signature' }),
          { status: 400, headers: { 'Content-Type': 'application/json' } }
        )
      }

      // Parse the AgentMail message from the envelope text
      let message: AgentMailMessage
      try {
        message = JSON.parse(envelope.text)
      } catch (error) {
        return new Response(
          JSON.stringify({ error: 'Invalid message format: not valid JSON' }),
          { status: 400, headers: { 'Content-Type': 'application/json' } }
        )
      }

      // Validate message structure
      if (!message.version || !message.from || !message.to || !message.timestamp || !message.body) {
        return new Response(
          JSON.stringify({ error: 'Invalid message format: missing required fields' }),
          { status: 400, headers: { 'Content-Type': 'application/json' } }
        )
      }

      // Verify that the envelope signer matches the message 'from' field
      if (envelope.signer !== message.from) {
        return new Response(
          JSON.stringify({ error: 'Message signer does not match from field' }),
          { status: 400, headers: { 'Content-Type': 'application/json' } }
        )
      }

      const senderPubkey = envelope.signer

      // Security checks
      if (this.security.checkRateLimit(senderPubkey)) {
        const info = this.security.getRateLimitInfo(senderPubkey)
        return new Response(
          JSON.stringify({ 
            error: 'Rate limit exceeded', 
            count: info.count,
            windowMs: this.security.getConfig().rateLimit.windowMs 
          }),
          { status: 429, headers: { 'Content-Type': 'application/json' } }
        )
      }

      if (!this.security.checkAllowlist(senderPubkey)) {
        return new Response(
          JSON.stringify({ error: 'Sender not in allowlist' }),
          { status: 403, headers: { 'Content-Type': 'application/json' } }
        )
      }

      if (!this.security.checkBodySize(message.body)) {
        return new Response(
          JSON.stringify({ error: 'Message body too large' }),
          { status: 413, headers: { 'Content-Type': 'application/json' } }
        )
      }

      // Generate message ID and store
      const messageId = `${Date.now()}-${message.from.slice(0, 8)}`
      const storedMessage: StoredMessage = {
        id: messageId,
        receivedAt: new Date().toISOString(),
        envelope,
        message,
        sender: envelope.signer
      }

      const filename = `${messageId}.json`
      const filepath = join(this.dataDir, filename)
      await writeFile(filepath, JSON.stringify(storedMessage, null, 2))

      console.log(`📨 Received message from ${message.from} to ${message.to}: "${message.subject || '(no subject)'}"`)

      return new Response(
        JSON.stringify({ 
          status: 'received', 
          id: messageId,
          receivedAt: storedMessage.receivedAt 
        }),
        { 
          status: 200, 
          headers: { 'Content-Type': 'application/json' } 
        }
      )

    } catch (error) {
      console.error('Error processing inbox message:', error)
      return new Response(
        JSON.stringify({ error: 'Internal server error' }),
        { status: 500, headers: { 'Content-Type': 'application/json' } }
      )
    }
  }

  async handleInboxGet(request: Request): Promise<Response> {
    try {
      const url = new URL(request.url)
      const limit = Math.min(parseInt(url.searchParams.get('limit') || '50'), 100)
      const since = url.searchParams.get('since')
      const from = url.searchParams.get('from')

      // Read all message files
      const files = await readdir(this.dataDir)
      const messageFiles = files.filter(f => f.endsWith('.json')).sort().reverse()

      const messages: StoredMessage[] = []
      
      for (const file of messageFiles.slice(0, limit)) {
        const filepath = join(this.dataDir, file)
        const content = await readFile(filepath, 'utf-8')
        const storedMessage: StoredMessage = JSON.parse(content)

        // Apply filters
        if (since && storedMessage.receivedAt <= since) continue
        if (from && storedMessage.message.from !== from) continue

        messages.push(storedMessage)
      }

      const isolated = url.searchParams.get('isolated') === 'true'

      return new Response(
        JSON.stringify({ 
          messages: messages.map(msg => ({
            id: msg.id,
            receivedAt: msg.receivedAt,
            from: msg.message.from,
            to: msg.message.to,
            timestamp: msg.message.timestamp,
            subject: msg.message.subject,
            body: isolated ? wrapUntrustedContent(msg.message) : msg.message.body
          })),
          count: messages.length 
        }),
        { 
          status: 200, 
          headers: { 'Content-Type': 'application/json' } 
        }
      )

    } catch (error) {
      console.error('Error fetching inbox messages:', error)
      return new Response(
        JSON.stringify({ error: 'Internal server error' }),
        { status: 500, headers: { 'Content-Type': 'application/json' } }
      )
    }
  }

  async handleRequest(request: Request): Promise<Response> {
    const url = new URL(request.url)
    
    // Enable CORS for development
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    }

    if (request.method === 'OPTIONS') {
      return new Response(null, { status: 200, headers: corsHeaders })
    }

    if (url.pathname === '/inbox') {
      if (request.method === 'POST') {
        const response = await this.handleInboxPost(request)
        return new Response(response.body, {
          status: response.status,
          headers: { ...Object.fromEntries(response.headers), ...corsHeaders }
        })
      } else if (request.method === 'GET') {
        const response = await this.handleInboxGet(request)
        return new Response(response.body, {
          status: response.status,
          headers: { ...Object.fromEntries(response.headers), ...corsHeaders }
        })
      }
    }

    if (url.pathname === '/health') {
      return new Response(
        JSON.stringify({ status: 'ok', timestamp: new Date().toISOString() }),
        { status: 200, headers: { 'Content-Type': 'application/json', ...corsHeaders } }
      )
    }

    if (url.pathname === '/config' && request.method === 'GET') {
      const config = this.security.getConfig()
      return new Response(
        JSON.stringify({ 
          security: {
            maxBodySize: config.maxBodySize,
            rateLimit: config.rateLimit,
            requireRegistered: config.requireRegistered,
            allowlistEnabled: !!(config.allowlist && config.allowlist.length > 0)
          } 
        }),
        { status: 200, headers: { 'Content-Type': 'application/json', ...corsHeaders } }
      )
    }

    return new Response(
      JSON.stringify({ error: 'Not found' }),
      { status: 404, headers: { 'Content-Type': 'application/json', ...corsHeaders } }
    )
  }

  async start() {
    await this.init()
    
    const server = Bun.serve({
      port: this.port,
      fetch: (request) => this.handleRequest(request),
    })

    console.log(`🚀 AgentMail inbox server running on port ${server.port}`)
    console.log(`📬 POST /inbox - receive messages`)
    console.log(`📮 GET /inbox - list messages`)
    console.log(`💚 GET /health - health check`)
    
    return server
  }
}

// Start server if run directly
if (import.meta.main) {
  const port = parseInt(process.env.PORT || '3000')
  const server = new InboxServer(port)
  await server.start()
}

export { InboxServer }