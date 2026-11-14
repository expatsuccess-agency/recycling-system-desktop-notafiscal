import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const ui = readFileSync(new URL('../src/renderer/index.tsx', import.meta.url), 'utf8');
test('novo mapeamento de teclas Fn V8 NotaFiscal', () => {
  for (const [label,key] of [['Pesagem','F1'],['Controle','F3'],['Caixa','F5'],['Gestão Financeira','F7'],['Configuração','F9']]) {
    assert.ok(ui.includes(`label:'${label}',key:'${key}'`),`${label} -> ${key}`);
  }
  assert.match(ui,/F1:'weigh',F3:'vendas',F7:'finance',F9:'config'/);
  assert.match(ui,/ev.key==='F5'\|\|ev.key==='F11'/);
  assert.match(ui,/setCaixaTab\(ev.key==='F5'\?'current':'previous'\)/);
  assert.ok(ui.includes('Caixa Atual <kbd>F5</kbd>'));
  assert.ok(ui.includes('Caixas Anteriores <kbd>F11</kbd>'));
  assert.ok(ui.includes('Anteriores <kbd>F11</kbd>'));
  assert.ok(!ui.includes("F2:'vendas'"));
  assert.ok(!ui.includes("F8:'finance'"));
  assert.ok(ui.includes("if(ev.key==='F12'"));
});
