import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
const auth=fs.readFileSync('src-tauri/src/auth.rs','utf8');const ui=fs.readFileSync('src/renderer/index.tsx','utf8');
test('all new passwords require exactly eight digits',()=>{assert.match(auth,/fn valid_pin\(pin:&str\)/);assert.match(auth,/pin\.len\(\)==8/);assert.match(auth,/is_ascii_digit/);assert.match(auth,/!valid_pin\(new\)/);assert.match(auth,/!valid_pin\(pwd\)/);});
test('all password screens use independent eye controls',()=>{assert.match(ui,/function PasswordField/);assert.match(ui,/aria-label=\{visible\?'Ocultar senha':'Mostrar senha'\}/);assert.match(ui,/EyeOff/);assert.equal((ui.match(/<PasswordField\b/g)||[]).length,5);});
test('application version unchanged',()=>{assert.equal(JSON.parse(fs.readFileSync('package.json','utf8')).version,'8.1.0');});
