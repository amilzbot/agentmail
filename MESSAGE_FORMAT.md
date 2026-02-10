# AgentMail Message Format Specification v1

## Overview

AgentMail uses **Solana Offchain Messages (sRFC 3)** as the signing envelope, with AgentMail-specific JSON payload as the message content. This provides:

- **Standard ed25519 signatures** verifiable by any Solana-compatible implementation
- **Version metadata** baked into the envelope (future-proof)
- **Signer verification** against Solana public keys
- **Cross-platform compatibility** (any system can verify signatures)

## Message Structure

### Offchain Message Envelope (sRFC 3)

The outer envelope follows Solana's standard offchain message format:

```typescript
interface OffchainMessageEnvelope {
  version: 0 | 1;
  address: Address; // Signer's Solana public key
  message: string;  // AgentMail JSON payload (see below)
  signature: string; // Base58-encoded ed25519 signature
  // ... other sRFC 3 fields
}
```

### AgentMail Payload (Inner Content)

The `message` field contains JSON-serialized AgentMail data:

```json
{
  "version": 1,
  "from": "CryptoKeyPublicKeyBase58String",
  "to": "RecipientPublicKeyBase58String", 
  "timestamp": "2026-02-10T00:30:00Z",
  "subject": "Optional subject line",
  "body": "# Hello\n\nThis is **markdown** content.\n\n- No HTML\n- No React\n- Just markdown",
  "thread_id": "optional-thread-identifier",
  "reply_to": "optional-message-id-being-replied-to"
}
```

## Field Specifications

### Required Fields

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `version` | number | AgentMail payload format version | Currently `1` |
| `from` | string | Sender's Solana public key (base58) | Must match envelope signer |
| `to` | string | Recipient's Solana public key (base58) | Valid base58 address |
| `timestamp` | string | ISO 8601 timestamp (UTC) | RFC 3339 format |
| `body` | string | Message content (markdown) | UTF-8, max 65KB |

### Optional Fields

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `subject` | string | Message subject/title | UTF-8, max 200 chars |
| `thread_id` | string | Thread identifier for grouping | UUID v4 recommended |
| `reply_to` | string | ID of message being replied to | UUID v4 or message hash |

## Signing Process

1. **Construct AgentMail payload** (JSON object with required fields)
2. **Canonicalize JSON** using `JSON.stringify()` with no extra whitespace
3. **Create offchain message** with payload as UTF-8 content
4. **Sign using ed25519** via @solana/kit's `signOffchainMessageEnvelope()`
5. **Result**: Complete signed envelope ready for transport

### Canonical JSON Serialization

To ensure signature consistency:

```typescript
const canonicalPayload = JSON.stringify(agentMailPayload, null, 0);
// No spaces, consistent key ordering via JSON.stringify default behavior
```

The canonical form is signed within the sRFC 3 envelope, NOT the raw JSON.

## Verification Process

1. **Parse offchain message envelope**
2. **Verify envelope signature** against sender's public key
3. **Decode inner AgentMail JSON**
4. **Validate required fields** and format
5. **Cross-check**: `payload.from` must equal `envelope.address`

## Size Limits

- **Small messages**: Use `OffchainMessageContentFormat.UTF8_1232_BYTES_MAX` (1.2KB)
- **Large messages**: Use `OffchainMessageContentFormat.UTF8_65535_BYTES_MAX` (64KB)
- **Body content**: Recommend ≤ 60KB to leave room for envelope overhead

## Version Strategy

- **Envelope version**: Handled by sRFC 3 (v0/v1 supported)
- **Payload version**: AgentMail semantic versioning
  - `version: 1` = This spec
  - `version: 2` = Future enhancements (encryption, attachments, etc.)
  - Backward compatibility required within major versions

## Security Notes

- **Signatures are NOT encrypted** — messages are signed but publicly readable
- **Replay protection**: Timestamps should be validated by receivers
- **Identity verification**: Always verify `from` field matches envelope signer
- **Registry validation**: Optionally verify sender is registered on-chain

## Reference Implementation

```typescript
import { 
  compileOffchainMessageEnvelope,
  signOffchainMessageEnvelope,
  OffchainMessageContentFormat
} from '@solana/kit';

// Create AgentMail message
const agentMailPayload = {
  version: 1,
  from: senderAddress,
  to: recipientAddress,
  timestamp: new Date().toISOString(),
  subject: "Hello!",
  body: "# Welcome\n\nThis is a test message."
};

// Compile and sign
const envelope = compileOffchainMessageEnvelope({
  version: 0,
  content: {
    format: OffchainMessageContentFormat.UTF8_1232_BYTES_MAX,
    text: JSON.stringify(agentMailPayload),
  },
  signatories: [senderAddress],
});

const signedEnvelope = await signOffchainMessageEnvelope(envelope, [signer]);
```

This signed envelope is ready for HTTPS POST transport to the recipient's inbox.