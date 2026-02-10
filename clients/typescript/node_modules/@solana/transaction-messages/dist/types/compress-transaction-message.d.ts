import { AccountLookupMeta, AccountMeta, Instruction } from '@solana/instructions';
import { AddressesByLookupTableAddress } from './addresses-by-lookup-table-address';
import { TransactionMessage } from './transaction-message';
type TransactionMessageNotLegacy = Exclude<TransactionMessage, {
    version: 'legacy';
}>;
type WidenInstructionAccounts<TInstruction extends Instruction> = TInstruction extends Instruction<infer TProgramAddress, infer TAccounts> ? Instruction<TProgramAddress, {
    [K in keyof TAccounts]: TAccounts[K] extends AccountMeta<infer TAddress> ? AccountLookupMeta<TAddress> | AccountMeta<TAddress> : TAccounts[K];
}> : TInstruction;
type WidenTransactionMessageInstructions<TTransactionMessage extends TransactionMessage> = TTransactionMessage extends {
    readonly instructions: readonly (infer TInstruction extends Instruction)[];
} ? Omit<TTransactionMessage, 'instructions'> & {
    readonly instructions: readonly WidenInstructionAccounts<TInstruction>[];
} : TTransactionMessage;
/**
 * Given a transaction message and a mapping of lookup tables to the addresses stored in them, this
 * function will return a new transaction message with the same instructions but with all non-signer
 * accounts that are found in the given lookup tables represented by an {@link AccountLookupMeta}
 * instead of an {@link AccountMeta}.
 *
 * This means that these accounts will take up less space in the compiled transaction message. This
 * size reduction is most significant when the transaction includes many accounts from the same
 * lookup table.
 *
 * @example
 * ```ts
 * import { address } from '@solana/addresses';
 * import {
 *     AddressesByLookupTableAddress,
 *     compressTransactionMessageUsingAddressLookupTables,
 * } from '@solana/transaction-messages';
 * import { fetchAddressLookupTable } from '@solana-program/address-lookup-table';
 *
 * const lookupTableAddress = address('4QwSwNriKPrz8DLW4ju5uxC2TN5cksJx6tPUPj7DGLAW');
 * const {
 *     data: { addresses },
 * } = await fetchAddressLookupTable(rpc, lookupTableAddress);
 * const addressesByAddressLookupTable: AddressesByLookupTableAddress = {
 *     [lookupTableAddress]: addresses,
 * };
 *
 * const compressedTransactionMessage = compressTransactionMessageUsingAddressLookupTables(
 *     transactionMessage,
 *     addressesByAddressLookupTable,
 * );
 * ```
 */
export declare function compressTransactionMessageUsingAddressLookupTables<TTransactionMessage extends TransactionMessageNotLegacy = TransactionMessageNotLegacy>(transactionMessage: TTransactionMessage, addressesByLookupTableAddress: AddressesByLookupTableAddress): TTransactionMessage | WidenTransactionMessageInstructions<TTransactionMessage>;
export {};
//# sourceMappingURL=compress-transaction-message.d.ts.map