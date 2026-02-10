/**
 * AgentMail Message Signing and Verification
 * 
 * Implements message signing using @solana/kit offchain messages (sRFC 3).
 * Uses V1 offchain messages for arbitrary-length UTF-8 content.
 */

import { 
  type Address,
  type OffchainMessageEnvelope,
  type OffchainMessageV1,
  type OffchainMessageSignatory,
  compileOffchainMessageV1Envelope,
  signOffchainMessageEnvelope,
  verifyOffchainMessageEnvelope,
  getAddressFromPublicKey,
  getOffchainMessageDecoder,
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
 * Signs an AgentMail message using Solana offchain messages (V1).
 * 
 * @param message - The AgentMail message to sign
 * @param senderKeyPair - The sender's Web Crypto CryptoKeyPair
 * @returns Signed message envelope
 */
export async function signAgentMailMessage(
  message: AgentMailMessage,
  senderKeyPair: CryptoKeyPair
): Promise<SignedAgentMailMessage> {
  const messageJson = JSON.stringify(message);
  
  // Derive the sender's address from the public key
  const senderAddress = await getAddressFromPublicKey(senderKeyPair.publicKey);
  
  // Build a V1 offchain message
  const offchainMessage: OffchainMessageV1 = {
    version: 1 as const,
    content: messageJson,
    requiredSignatories: [{ address: senderAddress } as OffchainMessageSignatory],
  };
  
  // Compile to an envelope (unsigned)
  const envelope = compileOffchainMessageV1Envelope(offchainMessage);
  
  // Sign the envelope with the sender's key pair
  const signedEnvelope = await signOffchainMessageEnvelope([senderKeyPair], envelope);
  
  return {
    envelope: signedEnvelope,
    payload: message,
  };
}

/**
 * Convenience function to create and sign a message in one step
 */
export async function createAndSignMessage(
  options: CreateMessageOptions,
  senderKeyPair: CryptoKeyPair
): Promise<SignedAgentMailMessage> {
  const message = createAgentMailMessage(options);
  return signAgentMailMessage(message, senderKeyPair);
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
    // Verify the offchain message signature (throws on failure)
    await verifyOffchainMessageEnvelope(envelope);
    
    // Decode the offchain message to extract the content string
    const decoded = getOffchainMessageDecoder().decode(envelope.content);
    
    // Extract the text content based on the message version
    let contentText: string;
    if ('content' in decoded && typeof decoded.content === 'string') {
      // V1 message: content is a plain string
      contentText = decoded.content;
    } else if ('content' in decoded && typeof decoded.content === 'object' && decoded.content !== null && 'text' in decoded.content) {
      // V0 message: content is { format, text }
      contentText = (decoded.content as { text: string }).text;
    } else {
      return { valid: false, error: 'Unable to extract message content' };
    }
    
    // Parse the message payload from the content
    let payload: AgentMailMessage;
    try {
      payload = JSON.parse(contentText) as AgentMailMessage;
    } catch {
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
    const signerAddresses = Object.keys(envelope.signatures);
    if (!signerAddresses.includes(payload.from)) {
      return { valid: false, error: 'Signer mismatch: envelope signer does not match message from field' };
    }
    
    return { valid: true, payload };
    
  } catch (error) {
    return { valid: false, error: `Verification error: ${error}` };
  }
}

/**
 * Serializes a signed message for transmission over HTTP
 */
export function serializeSignedMessage(signedMessage: SignedAgentMailMessage): string {
  return JSON.stringify(signedMessage.envelope);
}

/**
 * Deserializes a received message from HTTP
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
  messageId?: string;
  details?: unknown;
}

/**
 * Sends a signed message to a recipient's inbox URL
 */
export async function sendMessage(
  signedMessage: SignedAgentMailMessage,
  inboxUrl: string,
  timeoutMs: number = 10000
): Promise<SendMessageResponse> {
  try {
    if (!inboxUrl.startsWith('https://')) {
      return { 
        success: false, 
        error: 'Invalid inbox URL: must be HTTPS' 
      };
    }

    const messageData = serializeSignedMessage(signedMessage);
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

      let responseData: unknown = null;
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
 */
export async function createSignAndSendMessage(
  options: CreateMessageOptions,
  senderKeyPair: CryptoKeyPair,
  inboxUrl: string,
  timeoutMs: number = 10000
): Promise<SendMessageResponse> {
  try {
    const signedMessage = await createAndSignMessage(options, senderKeyPair);
    const result = await sendMessage(signedMessage, inboxUrl, timeoutMs);
    
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
