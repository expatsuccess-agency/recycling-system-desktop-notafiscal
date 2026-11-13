import React, {useState} from 'react';
import {open as openDialog} from '@tauri-apps/plugin-dialog';

type Api = (channel:string,...args:any[])=>Promise<any>;
export function A1E2Proof({api}:{api:Api}) {
  const [path,setPath]=useState('');
  const [password,setPassword]=useState('');
  const [busy,setBusy]=useState(false);
  const [out,setOut]=useState<any>(null);
  const [error,setError]=useState('');
  const run=async()=>{
    setBusy(true);setError('');setOut(null);
    try {setOut(await api('fiscal:a1:proof',{path,password}));}
    catch(e){setError(String(e));}
    finally {setPassword('');setBusy(false);}
  };
  return <section className="fiscalNotice" aria-label="Prova local do certificado A1">
    <h3>E2 · Prova local do certificado A1</h3>
    <p>Verifica posse da chave privada RSA por desafio aleatório. Não assina XML, não consulta ICP-Brasil e não emite NF-e.</p>
    <div className="formGrid">
      <label>Certificado A1 (.pfx / .p12)
        <input value={path} onChange={e=>setPath(e.target.value)} autoComplete="off"/>
      </label>
      <label>Senha do certificado (não gravada)
        <input type="password" value={password} onChange={e=>setPassword(e.target.value)} autoComplete="new-password"/>
      </label>
    </div>
    <div className="actionRow">
      <button type="button" onClick={async()=>{
        const p=await openDialog({multiple:false,directory:false,filters:[{name:'Certificado A1',extensions:['pfx','p12']}]});
        if(typeof p==='string')setPath(p);
      }}>Selecionar A1</button>
      <button type="button" className="primary" disabled={busy||!path||!password} onClick={()=>void run()}>
        {busy?'Verificando…':'Testar chave privada A1 (E2)'}
      </button>
    </div>
    {error&&<p role="alert">Falha E2: {error}</p>}
    {out&&<div role="status"><p>Assinatura do desafio: {out.assinatura_desafio_verificada?'verificada':'não verificada'}</p>
      <p>Certificado: {out.certificado_cn}</p>
      <p>SHA-256: {out.fingerprint_sha256}</p>
      <strong>NF-e e transmissão SEFAZ permanecem bloqueadas.</strong>
    </div>}
  </section>;
}
