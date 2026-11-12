import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const root=new URL('../',import.meta.url);
const read=p=>readFileSync(new URL(p,root),'utf8');
const ui=read('src/renderer/index.tsx'),backend=read('src-tauri/src/lib.rs');
const store=read('src-tauri/src/store.rs'),auth=read('src-tauri/src/auth.rs'),finance=read('src-tauri/src/finance.rs');
test('V5 preserves V4 license checks and adds backend authentication',()=>{
 assert.match(backend,/license::assert_valid/);assert.match(backend,/auth::current/);assert.match(backend,/ACESSO_NEGADO/);
});
test('admin first login has enforced password change',()=>{
 assert.match(auth,/hash\("admin"\)/);assert.match(auth,/must_change/);assert.match(auth,/Argon2/);
 assert.match(backend,/ALTERAÇÃO_DE_SENHA_OBRIGATÓRIA/);
});
test('employees cannot invoke admin channels via backend',()=>{
 assert.match(backend,/employee_allowed/);assert.match(backend,/me\.role!="admin"/);
 assert.match(backend,/channel=="checkout"/);
 assert.match(backend,/Some\("purchase"\)/);
 assert.match(store,/pub fn employee_articles/);
});
test('purchase pricing is backend-enforced and configured by product',()=>{
 assert.match(store,/price_editable/);assert.match(store,/PREÇO_NÃO_EDITÁVEL/);
 assert.match(ui,/Permitir alterar preço\/kg na Pesagem/);
});
test('financial data has immutable cost snapshot and expense categories',()=>{
 assert.match(store,/cost_per_kg/);assert.match(finance,/known_cost_cents/);
 assert.match(finance,/missingCostLines/);assert.match(finance,/expense_categories/);
});
test('Pesagem uses F12, keyboard focus and no client picker',()=>{
 assert.match(ui,/ev\.key==='F12'/);assert.match(ui,/productFocus\.current\?\.focus\(\)/);
 const start=ui.indexOf("{tab==='weigh'");const end=ui.indexOf("{tab==='vendas'",start);
 assert.doesNotMatch(ui.slice(start,end),/AutoSelect label="Cliente"/);
});
test('Finance is accessible only to admin in UI and Rust',()=>{
 assert.match(ui,/Gestão Financeira/);assert.match(ui,/isAdmin&&tab==='finance'/);
 assert.match(backend,/c\.starts_with\("finance:"\)/);
});
