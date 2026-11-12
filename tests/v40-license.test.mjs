import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const root=new URL('../',import.meta.url);
const read=p=>readFileSync(new URL(p,root),'utf8');
test('backend checks license before business command dispatch',()=>{
 const src=read('src-tauri/src/lib.rs');
 assert.match(src,/license::assert_valid\(&app\)\?/);
 assert.match(src,/"license:install"/);
});
test('V4 license is signed locally and machine-bound',()=>{
 const src=read('src-tauri/src/license.rs');
 assert.match(src,/key\.verify\(&payload_bytes,&signature\)/);
 assert.match(src,/p\.machine_id!=machine_id\(\)\?/);
 assert.match(src,/LICENÇA_EXPIRADA/);
});
test('no embedded developer bypass or issuer private key',()=>{
 const src=read('src-tauri/src/license.rs');
 assert.doesNotMatch(src,/private-key\.pem|LXO_DEV_LICENSE_KEY/);
 const build=read('src-tauri/build.rs');
 assert.match(build,/panic!/);
});
