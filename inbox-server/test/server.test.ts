import { expect, test, describe, beforeAll, afterAll } from "bun:test"
import { InboxServer } from "../src/server.js"

describe("AgentMail Inbox Server", () => {
  let server: any
  let baseUrl: string

  beforeAll(async () => {
    const inboxServer = new InboxServer(0, './test-data') // Use port 0 for random port
    server = await inboxServer.start()
    baseUrl = `http://localhost:${server.port}`
  })

  afterAll(() => {
    server?.stop()
  })

  test("health check endpoint", async () => {
    const response = await fetch(`${baseUrl}/health`)
    expect(response.status).toBe(200)
    
    const data = await response.json()
    expect(data.status).toBe("ok")
    expect(data.timestamp).toBeDefined()
  })

  test("config endpoint", async () => {
    const response = await fetch(`${baseUrl}/config`)
    expect(response.status).toBe(200)
    
    const data = await response.json()
    expect(data.security).toBeDefined()
    expect(data.security.maxBodySize).toBe(32768) // 32KB
    expect(data.security.rateLimit).toBeDefined()
  })

  test("empty inbox returns empty messages", async () => {
    const response = await fetch(`${baseUrl}/inbox`)
    expect(response.status).toBe(200)
    
    const data = await response.json()
    expect(data.messages).toEqual([])
    expect(data.count).toBe(0)
  })

  test("invalid message returns 500", async () => {
    const response = await fetch(`${baseUrl}/inbox`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ invalid: 'message' })
    })
    expect(response.status).toBe(500)
    
    const data = await response.json()
    expect(data.error).toBe("Internal server error")
  })

  test("404 for unknown endpoints", async () => {
    const response = await fetch(`${baseUrl}/unknown`)
    expect(response.status).toBe(404)
    
    const data = await response.json()
    expect(data.error).toBe("Not found")
  })
})