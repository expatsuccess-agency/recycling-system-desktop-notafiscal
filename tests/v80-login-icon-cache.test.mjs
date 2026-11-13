import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const ui=readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const css=readFileSync(new URL('../src/renderer/style.css',import.meta.url),'utf8');
const packageJson=JSON.parse(readFileSync(new URL('../package.json',import.meta.url),'utf8'));
test('version 8 appears on login and in the application',()=>{
 assert.equal(packageJson.version,'8.1.0');
 assert.match(ui,/className="loginVersion">V8\.1\.0/);
 assert.match(ui,/Versão 8\.0\.0/);
});
test('login uses spinner while authenticating and handles errors',()=>{
 assert.match(ui,/if\(loggingIn\)return/);
 assert.match(ui,/setLoggingIn\(true\)/);
 assert.match(ui,/finally\{setLoggingIn\(false\)\}/);
 assert.match(ui,/LoaderCircle className="loginSpinner"/);
 assert.match(ui,/disabled=\{loggingIn\}/);
 assert.match(css,/@keyframes loginSpin/);
});
test('icon picker has search, categories, selection and closes on outside click',()=>{
 for(const term of ['function MaterialIconPicker','Buscar ícone ou categoria','iconPickerCategories','iconPickerGrid',"document.addEventListener('pointerdown'",'aria-pressed={value===ic}','onChange(ic)']) assert.ok(ui.includes(term),term);
 assert.match(css,/\.iconPickerPanel\{/);
});
test('shared build cache launcher exists and preserves version-specific sources',()=>{
 const shell=readFileSync(new URL('../lancer-avec-cache.sh',import.meta.url),'utf8');
 assert.match(shell,/CARGO_TARGET_DIR/);
 assert.match(shell,/npm_config_cache/);
 assert.match(shell,/npm ci --prefer-offline/);
});
