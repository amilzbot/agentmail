/**
 * Generates TypeScript and Rust clients from the Codama IDL.
 */

import type { AnchorIdl } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from '@codama/renderers-rust';
import fs from 'fs';
import path from 'path';

import { createAgentMailCodamaBuilder } from './lib/agentmail-codama-builder';
import { preserveConfigFiles } from './lib/utils';

const projectRoot = path.join(__dirname, '..');
const idlDir = path.join(projectRoot, 'idl');
const agentmailIdl = JSON.parse(
    fs.readFileSync(path.join(idlDir, 'agentmail.json'), 'utf-8'),
) as AnchorIdl;
const rustClientsDir = path.join(__dirname, '..', 'clients', 'rust');
const typescriptClientsDir = path.join(__dirname, '..', 'clients', 'typescript');

const agentmailCodama = createAgentMailCodamaBuilder(agentmailIdl)
    .appendAccountDiscriminator()
    .appendPdaDerivers()
    .setInstructionAccountDefaultValues()
    .updateInstructionBumps()
    .build();

// Preserve configuration files during generation
const configPreserver = preserveConfigFiles(typescriptClientsDir, rustClientsDir);

// Generate Rust client
void agentmailCodama.accept(
    renderRustVisitor(path.join(rustClientsDir, 'src', 'generated'), {
        crateFolder: rustClientsDir,
        deleteFolderBeforeRendering: true,
        formatCode: true,
    }),
);

// Generate TypeScript client
void agentmailCodama.accept(
    renderJavaScriptVisitor(path.join(typescriptClientsDir, 'src', 'generated'), {
        deleteFolderBeforeRendering: true,
        formatCode: true,
    }),
);

// Restore configuration files after generation
configPreserver.restore();
