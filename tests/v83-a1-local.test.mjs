import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const rust=fs.readFileSync(new URL('../src-tauri/src/a1.rs', import.meta.url),'utf8');
const backend=fs.readFileSync(new URL('../src-tauri/src/fiscal.rs', import.meta.url),'utf8');
const ui=fs.readFileSync(new URL('../src/renderer/index.tsx', import.meta.url),'utf8');
test('A1 password is read from request only, not stored in database',()=>{
 assert.match(backend,/"fiscal:a1:inspect"/);
 assert.match(rust,/"-passin","stdin"/);
 assert.doesNotMatch(rust,/"-passin","pass:/);
 assert.match(ui,/setA1Password\(''\)/);
});
test('A1 check does not authorize NF-e or claim ICP-Brasil trust',()=>{
 assert.match(rust,/"cnpj_verificado":false/);
 assert.match(rust,/"cadeia_icp_brasil_verificada":false/);
 assert.match(rust,/"revogacao_verificada":false/);
 assert.match(rust,/"emissao_habilitada":false/);
 assert.match(backend,/EMISSAO_SEFAZ_NAO_IMPLEMENTADA_NAO_TRANSMITIDO/);
});
