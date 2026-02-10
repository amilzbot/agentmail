import { Codama, createFromJson } from 'codama';
import {
    appendAccountDiscriminator,
    appendPdaDerivers,
    setInstructionAccountDefaultValues,
    updateInstructionBumps,
} from './updates';

/**
 * Builder for applying Codama IDL transformations before client generation.
 */
export class AgentMailCodamaBuilder {
    private codama: Codama;

    constructor(agentmailIdl: unknown) {
        const idlJson =
            typeof agentmailIdl === 'string' ? agentmailIdl : JSON.stringify(agentmailIdl);
        this.codama = createFromJson(idlJson);
    }

    appendAccountDiscriminator(): this {
        this.codama = appendAccountDiscriminator(this.codama);
        return this;
    }

    appendPdaDerivers(): this {
        this.codama = appendPdaDerivers(this.codama);
        return this;
    }

    setInstructionAccountDefaultValues(): this {
        this.codama = setInstructionAccountDefaultValues(this.codama);
        return this;
    }

    updateInstructionBumps(): this {
        this.codama = updateInstructionBumps(this.codama);
        return this;
    }

    build(): Codama {
        return this.codama;
    }
}

export function createAgentMailCodamaBuilder(agentmailIdl: unknown): AgentMailCodamaBuilder {
    return new AgentMailCodamaBuilder(agentmailIdl);
}
