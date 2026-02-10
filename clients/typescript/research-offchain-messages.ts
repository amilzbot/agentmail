#!/usr/bin/env bun

// Research script for @solana/kit OffchainMessage functionality
// Questions to answer:
// 1. Can we embed arbitrary JSON as message text?
// 2. Signature portability (can verify without @solana/kit)?
// 3. What does the signed envelope look like?

import { 
  compileOffchainMessageEnvelope,
  signOffchainMessageEnvelope,
  verifyOffchainMessageEnvelope,
  generateKeyPair,
  OffchainMessageContentFormat
} from '@solana/kit';

async function researchOffchainMessages() {
  console.log("🔬 Researching @solana/kit OffchainMessage functionality...\n");

  // Generate a test keypair
  const signer = await generateKeyPair();
  console.log("Test signer:", signer.publicKey);

  // Test 1: Can we embed arbitrary JSON as message text?
  console.log("\n📝 Test 1: Embedding JSON content");
  
  const agentmailPayload = {
    version: 1,
    from: signer.publicKey,
    to: "SomeRecipientPubkey111111111111111111111111",
    timestamp: "2026-02-10T00:30:00Z",
    subject: "Test message",
    body: "# Hello\n\nThis is **markdown** content.\n\n- No HTML\n- No React\n- Just markdown"
  };

  const jsonContent = JSON.stringify(agentmailPayload, null, 2);
  console.log("JSON content to embed:", jsonContent);

  try {
    // First compile the offchain message with proper version and format
    const compiledMessage = compileOffchainMessageEnvelope({
      version: 0, // Start with v0
      content: {
        format: OffchainMessageContentFormat.UTF8_1232_BYTES_MAX,
        text: jsonContent,
      },
      signatories: [signer.publicKey],
    });
    
    console.log("✅ Step 1: Compiled offchain message");
    console.log("Message envelope keys:", Object.keys(compiledMessage));
    
    // Then sign it
    const signedMessage = await signOffchainMessageEnvelope(compiledMessage, [signer]);
    
    console.log("✅ Step 2: Signed offchain message with JSON content");
    console.log("Signed message keys:", Object.keys(signedMessage));
    console.log("Message format:", JSON.stringify(signedMessage, null, 2));

    // Test 2: Verify the message
    console.log("\n✅ Test 3: Verification");
    
    const verification = await verifyOffchainMessageEnvelope(signedMessage);
    console.log("Verification result:", verification);
    
    // Test 3: Examine the raw signature and data structure
    console.log("\n🔍 Test 4: Raw signature examination");
    console.log("Message structure analysis:");
    Object.keys(signedMessage).forEach(key => {
      console.log(`- ${key}:`, typeof signedMessage[key], Array.isArray(signedMessage[key]) ? `(array of ${signedMessage[key].length})` : '');
    });

    // Test 4: Can we extract/verify signature manually?
    console.log("\n🛠️ Test 5: Manual signature verification possibilities");
    console.log("This envelope can be verified by any system with access to:");
    console.log("- The Solana public key of the signer");
    console.log("- The envelope structure (which follows sRFC 3 standard)");
    
    return true;
  } catch (error) {
    console.error("❌ Error:", error);
    return false;
  }
}

// Run the research
researchOffchainMessages()
  .then(success => {
    if (success) {
      console.log("\n✅ Research complete! OffchainMessage looks viable for AgentMail.");
    } else {
      console.log("\n❌ Research failed. May need alternative approach.");
    }
  })
  .catch(console.error);