/**
 * Security middleware and rate limiting for AgentMail inbox server
 */

interface RateLimitEntry {
  count: number
  windowStart: number
}

interface SecurityConfig {
  maxBodySize: number // bytes
  rateLimit: {
    windowMs: number
    maxRequests: number
  }
  requireRegistered: boolean
  allowlist?: string[] // pubkeys
}

export class SecurityManager {
  private rateLimitCache = new Map<string, RateLimitEntry>()
  private config: SecurityConfig

  constructor(config: Partial<SecurityConfig> = {}) {
    this.config = {
      maxBodySize: 32 * 1024, // 32KB
      rateLimit: {
        windowMs: 60 * 60 * 1000, // 1 hour
        maxRequests: 10
      },
      requireRegistered: false,
      ...config
    }
  }

  /**
   * Check if request should be rate limited
   */
  checkRateLimit(senderPubkey: string): boolean {
    const now = Date.now()
    const windowStart = now - this.config.rateLimit.windowMs
    
    const entry = this.rateLimitCache.get(senderPubkey)
    
    if (!entry || entry.windowStart < windowStart) {
      // Start new window
      this.rateLimitCache.set(senderPubkey, {
        count: 1,
        windowStart: now
      })
      return false
    }

    if (entry.count >= this.config.rateLimit.maxRequests) {
      return true // Rate limited
    }

    entry.count++
    return false
  }

  /**
   * Check if sender is in allowlist (if configured)
   */
  checkAllowlist(senderPubkey: string): boolean {
    if (!this.config.allowlist || this.config.allowlist.length === 0) {
      return true // No allowlist configured
    }
    return this.config.allowlist.includes(senderPubkey)
  }

  /**
   * Validate message body size
   */
  checkBodySize(body: string): boolean {
    return Buffer.byteLength(body, 'utf8') <= this.config.maxBodySize
  }

  /**
   * Clean up old rate limit entries
   */
  cleanup() {
    const now = Date.now()
    const cutoff = now - this.config.rateLimit.windowMs

    for (const [key, entry] of this.rateLimitCache.entries()) {
      if (entry.windowStart < cutoff) {
        this.rateLimitCache.delete(key)
      }
    }
  }

  /**
   * Get current rate limit info for a sender
   */
  getRateLimitInfo(senderPubkey: string) {
    const entry = this.rateLimitCache.get(senderPubkey)
    if (!entry) return { count: 0, remaining: this.config.rateLimit.maxRequests }
    
    const remaining = Math.max(0, this.config.rateLimit.maxRequests - entry.count)
    return { count: entry.count, remaining }
  }

  getConfig() {
    return { ...this.config }
  }
}

/**
 * Content isolation wrapper - wraps untrusted message content
 */
export function wrapUntrustedContent(
  message: { from: string; subject?: string; body: string },
  boundaryNonce?: string
): string {
  const nonce = boundaryNonce || generateBoundaryNonce()
  
  return `⚠️ EXTERNAL UNTRUSTED AGENT MESSAGE [boundary:${nonce}]
From: ${message.from}
Subject: ${message.subject || '(no subject)'}
---
${message.body}
---
END EXTERNAL AGENT MESSAGE [boundary:${nonce}]`
}

function generateBoundaryNonce(): string {
  return Math.random().toString(36).substring(2, 10)
}

// Periodic cleanup - run every 10 minutes
setInterval(() => {
  // This would be handled by a global SecurityManager instance
}, 10 * 60 * 1000)