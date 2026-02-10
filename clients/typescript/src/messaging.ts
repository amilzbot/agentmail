/**
 * AgentMail Message Signing and Verification
 * 
 * Implements message signing using @solana/kit offchain messages (sRFC 3).
 */

import { 
  type Address,
  type CryptoKeyPair,
  compileOffchainMessageEnvelope,
  signOffchainMessageEnvelope,
  verifyOffchainMessageEnvelope,
  type OffchainMessageEnvelope,
  OffchainMessageContentFormat,
} from '@solana/kit';

/**
 * AgentMail message payload structure
 */
export interface AgentMailMessage {
  version: number;
  from: Address;
  to: Address;
  timestamp: string; // ISO 8601 timestamp
  subject?: string;
  body: string; // Markdown content
  thread_id?: string;
  reply_to?: string;
}

/**
 * Options for creating an AgentMail message
 */
export interface CreateMessageOptions {
  from: Address;
  to: Address;
  subject?: string;
  body: string;
  thread_id?: string;
  reply_to?: string;
  timestamp?: string; // Optional, defaults to now
}

/**
 * Signed AgentMail message with envelope
 */
export interface SignedAgentMailMessage {
  envelope: OffchainMessageEnvelope;
  payload: AgentMailMessage;
}

/**
 * Creates an AgentMail message payload
 * 
 * @param options - Message creation options
 * @returns AgentMail message payload
 */
export function createAgentMailMessage(options: CreateMessageOptions): AgentMailMessage {
  return {
    version: 1,
    from: options.from,
    to: options.to,
    timestamp: options.timestamp ?? new Date().toISOString(),
    subject: options.subject,
    body: options.body,
    thread_id: options.thread_id,
    reply_to: options.reply_to,
  };
}

/**
 * Signs an AgentMail message using Solana offchain messages
 * 
 * @param message - The AgentMail message to sign
 * @param senderKeypair - The sender's Solana keypair
 * @returns Signed message envelope
 */
export async function signAgentMailMessage(
  message: AgentMailMessage,
  senderKeypair: CryptoKeyPair
): Promise<SignedAgentMailMessage> {
  // Serialize the message payload to JSON
  const messageJson = JSON.stringify(message);
  
  // Compile the offchain message envelope
  const envelope = await compileOffchainMessageEnvelope({
    content: messageJson,
    contentFormat: OffchainMessageContentFormat.UTF8_65535_BYTES_MAX,
    address: message.from,
  });
  
  // Sign the envelope
  const signedEnvelope = await signOffchainMessageEnvelope({
    envelope,
    keyPair: senderKeypair,
  });
  
  return {
    envelope: signedEnvelope,
    payload: message,
  };
}

/**
 * Convenience function to create and sign a message in one step
 * 
 * @param options - Message creation options  
 * @param senderKeypair - The sender's Solana keypair
 * @returns Signed message envelope
 */
export async function createAndSignMessage(
  options: CreateMessageOptions,
  senderKeypair: CryptoKeyPair
): Promise<SignedAgentMailMessage> {
  const message = createAgentMailMessage(options);
  return signAgentMailMessage(message, senderKeypair);
}

/**
 * Verifies a signed AgentMail message
 * 
 * @param envelope - The signed message envelope
 * @returns Verification result with parsed payload if valid
 */
export async function verifyAgentMailMessage(
  envelope: OffchainMessageEnvelope
): Promise<{ valid: boolean; payload?: AgentMailMessage; error?: string }> {
  try {
    // Verify the offchain message signature
    const isValid = await verifyOffchainMessageEnvelope(envelope);
    
    if (!isValid) {
      return { valid: false, error: 'Invalid signature' };
    }
    
    // Parse the message payload
    let payload: AgentMailMessage;
    try {
      payload = JSON.parse(envelope.message) as AgentMailMessage;
    } catch (error) {
      return { valid: false, error: 'Invalid JSON payload' };
    }
    
    // Basic validation
    if (payload.version !== 1) {
      return { valid: false, error: `Unsupported message version: ${payload.version}` };
    }
    
    if (!payload.from || !payload.to || !payload.body) {
      return { valid: false, error: 'Missing required fields (from, to, body)' };
    }
    
    // Verify that the signer matches the 'from' field
    if (envelope.address !== payload.from) {
      return { valid: false, error: 'Signer mismatch: envelope signer does not match message from field' };
    }
    
    return { valid: true, payload };
    
  } catch (error) {
    return { valid: false, error: `Verification error: ${error}` };
  }
}

/**
 * Serializes a signed message for transmission over HTTP
 * 
 * @param signedMessage - The signed message to serialize
 * @returns JSON string ready for HTTP transmission
 */
export function serializeSignedMessage(signedMessage: SignedAgentMailMessage): string {
  return JSON.stringify(signedMessage.envelope);
}

/**
 * Deserializes a received message from HTTP
 * 
 * @param data - JSON string received over HTTP
 * @returns Parsed envelope ready for verification
 */
export function deserializeSignedMessage(data: string): OffchainMessageEnvelope {
  return JSON.parse(data) as OffchainMessageEnvelope;
}

/**
 * Response from sending a message
 */
export interface SendMessageResponse {
  success: boolean;
  error?: string;
  details?: any;
}

/**
 * Sends a signed message to a recipient's inbox URL
 * 
 * @param signedMessage - The signed message to send
 * @param inboxUrl - The recipient's inbox URL
 * @param timeoutMs - Request timeout in milliseconds (default: 10000)
 * @returns Send result
 */
export async function sendMessage(
  signedMessage: SignedAgentMailMessage,
  inboxUrl: string,
  timeoutMs: number = 10000
): Promise<SendMessageResponse> {
  try {
    // Validate inbox URL
    if (!inboxUrl.startsWith('https://')) {
      return { 
        success: false, 
        error: 'Invalid inbox URL: must be HTTPS' 
      };
    }

    // Serialize the message for transmission
    const messageData = serializeSignedMessage(signedMessage);

    // Create fetch request with timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const response = await fetch(inboxUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'User-Agent': 'AgentMail/1.0',
        },
        body: messageData,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorText = await response.text().catch(() => 'Unknown error');
        return {
          success: false,
          error: `HTTP ${response.status}: ${response.statusText}`,
          details: { status: response.status, body: errorText },
        };
      }

      // Try to parse response body
      let responseData: any = null;
      try {
        const responseText = await response.text();
        if (responseText) {
          responseData = JSON.parse(responseText);
        }
      } catch {
        // Non-JSON response is okay
      }

      return {
        success: true,
        details: responseData,
      };

    } finally {
      clearTimeout(timeoutId);
    }

  } catch (error) {
    if (error instanceof Error) {
      if (error.name === 'AbortError') {
        return {
          success: false,
          error: `Request timeout after ${timeoutMs}ms`,
        };
      }
      return {
        success: false,
        error: `Network error: ${error.message}`,
      };
    }
    return {
      success: false,
      error: 'Unknown error occurred',
    };
  }
}

/**
 * Convenience function to create, sign, and send a message
 * 
 * @param options - Message creation options
 * @param senderKeypair - The sender's Solana keypair
 * @param inboxUrl - The recipient's inbox URL
 * @param timeoutMs - Request timeout in milliseconds (default: 10000)
 * @returns Send result with message details
 */
export async function createSignAndSendMessage(
  options: CreateMessageOptions,
  senderKeypair: CryptoKeyPair,
  inboxUrl: string,
  timeoutMs: number = 10000
): Promise<SendMessageResponse & { messageId?: string }> {
  try {
    // Create and sign the message
    const signedMessage = await createAndSignMessage(options, senderKeypair);
    
    // Send the message
    const result = await sendMessage(signedMessage, inboxUrl, timeoutMs);
    
    // Add message ID for tracking
    return {
      ...result,
      messageId: signedMessage.payload.timestamp + '-' + signedMessage.payload.from.slice(-8),
    };
    
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}