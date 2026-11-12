import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const src=readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const css=readFileSync(new URL('../src/renderer/style.css',import.meta.url),'utf8');
const pdf=readFileSync(new URL('../src-tauri/src/pdf.rs',import.meta.url),'utf8');
test('Enter on Peso adds purchase line',()=>{assert.match(src,/onKeyDown=\{e=>\{if\(e\.key==='Enter'&&!e\.repeat\)/);assert.match(src,/addLine\(\)/);assert.doesNotMatch(src,/Adicionar outro item/)});
test('Timeline typography reduced',()=>{assert.match(css,/\.timelineEvent\{font-size:9px/);assert.match(css,/\.timelineInfo\{font-size:9px/)});
test('PDF handles Portuguese WinAnsi and new brand',()=>{assert.match(pdf,/WinAnsiEncoding/);assert.match(pdf,/winansi_pdf_string/);assert.match(pdf,/RECYCLING SYSTEM INDUSTRIAL/);assert.doesNotMatch(pdf,/c}else\{'\?'\}/)});
