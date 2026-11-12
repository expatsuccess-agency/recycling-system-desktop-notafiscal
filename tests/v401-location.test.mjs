import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const read=p=>readFileSync(new URL('../'+p,import.meta.url),'utf8');
test('source input and external location are distinct',()=>{
 const ui=read('src/renderer/index.tsx');
 assert.match(ui,/license:use-external/);assert.match(ui,/license:install/);
 assert.match(ui,/openDialog/);assert.match(ui,/Local configurado/);
});
test('external location is signed, checked and stored without key leakage',()=>{
 const s=read('src-tauri/src/license.rs');
 assert.match(s,/pub fn use_external/);assert.match(s,/check_content\(&content\)/);
 assert.match(s,/checked_file\(&license_path\(app\)\?\)/);
 assert.match(s,/license-location.json/);
 assert.doesNotMatch(s,/SigningKey|private-key.pem/);
});
