/**
 * Demo: Using the WASM bindings from Node.js
 *
 * Run after building WASM:
 *   wasm-pack build crates/data-ingestion-wasm --target nodejs --out-dir dist/wasm-nodejs
 *   node examples/demo.js
 */

const fs = require('fs');
const path = require('path');

// Try to load the WASM module
let wasmModule;
try {
    wasmModule = require('../dist/wasm-nodejs/data_ingestion_wasm.js');
} catch (e) {
    console.error('ERROR: WASM module not found.');
    console.error('Build it first with: wasm-pack build crates/data-ingestion-wasm --target nodejs --out-dir dist/wasm-nodejs');
    process.exit(1);
}

const { ContractEngine } = wasmModule;

// Create engine
const engine = new ContractEngine();
engine.set_owner('data-team');
engine.set_domain('e-commerce');
engine.set_enrich_pii(true);

const examplesDir = __dirname;

// Example 1: JSON Schema
console.log('\n=== Example 1: JSON Schema ===');
const schemaContent = fs.readFileSync(path.join(examplesDir, 'sample_json_schema.json'));
const schemaBytes = new Uint8Array(schemaContent);

try {
    const contractJson = engine.process(schemaBytes, 'json_schema', 'sample_json_schema.json');
    const contract = JSON.parse(contractJson);

    console.log(`Contract name: ${contract.name}`);
    console.log(`Fields (${contract.fields.length}):`);
    for (const field of contract.fields) {
        const piiMarker = field.pii ? ' [PII]' : '';
        console.log(`  - ${field.name} (${JSON.stringify(field.logical_type)}) nullable=${field.nullable}${piiMarker}`);
    }

    // CSV output
    const csvOutput = engine.process_to_format(schemaBytes, 'json_schema', 'sample_json_schema.json', 'csv');
    console.log('\nCSV Output:');
    console.log(csvOutput);

} catch (e) {
    console.error('Error processing JSON Schema:', e);
}

// Example 2: Validation
console.log('\n=== Example 2: Validation ===');
const contractJson = engine.process(schemaBytes, 'json_schema', 'sample_json_schema.json');
const validationJson = engine.validate_contract_json(contractJson);
const validation = JSON.parse(validationJson);
console.log(`Valid: ${validation.valid}`);
console.log(`Warnings: ${JSON.stringify(validation.warnings)}`);
console.log(`Errors: ${JSON.stringify(validation.errors)}`);

console.log('\n=== All examples complete ===');
