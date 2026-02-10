/**
 * Generates TypeScript client only from the Codama IDL.
 */

import type { AnchorIdl } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import fs from 'fs';
import path from 'path';

import { createAgentMailCodamaBuilder } from './lib/agentmail-codama-builder';

const projectRoot = path.join(__dirname, '..');
const idlDir = path.join(projectRoot, 'idl');
const agentmailIdl = JSON.parse(
    fs.readFileSync(path.join(idlDir, 'agentmail.json'), 'utf-8'),
) as AnchorIdl;
const typescriptClientsDir = path.join(__dirname, '..', 'clients', 'typescript');

const agentmailCodama = createAgentMailCodamaBuilder(agentmailIdl)
    .appendAccountDiscriminator()
    .appendPdaDerivers()
    .setInstructionAccountDefaultValues()
    .updateInstructionBumps()
    .build();

// Generate TypeScript client only
void agentmailCodama.accept(
    renderJavaScriptVisitor(path.join(typescriptClientsDir, 'src', 'generated'), {
        deleteFolderBeforeRendering: true,
        formatCode: false, // Skip formatting to avoid permission issues
    }),
);

console.log('✅ TypeScript client generated successfully!');