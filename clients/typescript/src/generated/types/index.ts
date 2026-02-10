/**
 * Custom string type for AgentMail.
 * Borsh-style: u32le length prefix + UTF-8 bytes.
 */
import {
  addEncoderSizePrefix,
  addDecoderSizePrefix,
  combineCodec,
  getUtf8Encoder,
  getUtf8Decoder,
  getU32Encoder,
  getU32Decoder,
  type Encoder,
  type Decoder,
  type Codec,
} from '@solana/kit';

export type String = string;
export type StringArgs = string;

export function getStringEncoder(): Encoder<StringArgs> {
  return addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder());
}

export function getStringDecoder(): Decoder<String> {
  return addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder());
}

export function getStringCodec(): Codec<StringArgs, String> {
  return combineCodec(getStringEncoder(), getStringDecoder());
}
