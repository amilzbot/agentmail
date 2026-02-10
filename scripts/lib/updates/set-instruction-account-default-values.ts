import {
    Codama,
    pdaValueNode,
    pdaSeedValueNode,
    accountValueNode,
    publicKeyValueNode,
    pdaLinkNode,
    setInstructionAccountDefaultValuesVisitor,
} from 'codama';

const AGENTMAIL_PROGRAM_ID = 'AMz2ybwRihFL9X4igLBtqNBEe9qqb4yUvjwNwEaPjNiX';
const SYSTEM_PROGRAM_ID = '11111111111111111111111111111111';

/**
 * Sets default values for common instruction accounts (program IDs, PDAs).
 */
export function setInstructionAccountDefaultValues(pinocchioCounterCodama: Codama): Codama {
    pinocchioCounterCodama.update(
        setInstructionAccountDefaultValuesVisitor([
            // Global Constants
            {
                account: 'pinocchioCounterProgram',
                defaultValue: publicKeyValueNode(AGENTMAIL_PROGRAM_ID),
            },
            {
                account: 'systemProgram',
                defaultValue: publicKeyValueNode(SYSTEM_PROGRAM_ID),
            },

            // PDA Derivations
            {
                account: 'counter',
                defaultValue: pdaValueNode(pdaLinkNode('counter'), [
                    pdaSeedValueNode('authority', accountValueNode('authority')),
                ]),
            },
            {
                account: 'eventAuthority',
                defaultValue: pdaValueNode(pdaLinkNode('eventAuthority'), []),
            },
        ]),
    );
    return pinocchioCounterCodama;
}
