import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const ui=fs.readFileSync('src/renderer/index.tsx','utf8');
const css=fs.readFileSync('src/renderer/style.css','utf8');
test('product list before form fields',()=>{const start=ui.indexOf("{configTab==='products'&&<>");const end=ui.indexOf('{configTab===\'control\'',start);const region=ui.slice(start,end);assert.ok(region.indexOf('Lista de produtos')<region.indexOf('<label>Código'));assert.ok(region.indexOf('articles.map(p=>')<region.indexOf('<label>Código'));});
test('compact icon picker with searchable palette',()=>{assert.match(ui,/className="iconPickerTrigger"/);assert.match(ui,/className="iconPickerGrid"/);assert.match(css,/\.iconPickerGrid\{/);});
test('version six',()=>{assert.equal(JSON.parse(fs.readFileSync('package.json','utf8')).version,'8.1.0');});
