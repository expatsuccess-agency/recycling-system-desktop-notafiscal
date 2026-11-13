import React,{useEffect,useState} from 'react';
type Obj=Record<string,any>;
export function DebugTerminal({api,onClose}:{api:(channel:string,...args:any[])=>Promise<any>,onClose:()=>void}){
 const [rows,setRows]=useState<Obj[]>([]),[filter,setFilter]=useState('TODOS'),[error,setError]=useState('');
 const refresh=()=>void api('debug:events').then(setRows).catch(e=>setError(String(e)));
 useEffect(()=>{refresh();const id=window.setInterval(refresh,3000);return()=>clearInterval(id)},[]);
 const shown=rows.filter(r=>filter==='TODOS'||r.category===filter);
 const exportText=()=>{const lines=shown.map(r=>`${r.at} [${r.category}] ${r.method} ${r.endpoint} ${r.status} ${r.duration_ms}ms ${r.detail}`);const blob=new Blob([lines.join('\n')],{type:'text/plain'});const url=URL.createObjectURL(blob);const a=document.createElement('a');a.href=url;a.download='recycling-debug-sanitizado.log';a.click();URL.revokeObjectURL(url)};
 return <div className="debugOverlay" role="presentation"><section className="debugTerminal" role="dialog" aria-modal="true" aria-label="Terminal de depuração"><header><strong>▣ DEBUG · Histórico de comunicações</strong><button onClick={onClose} aria-label="Fechar debug">Fechar ×</button></header>
 <nav><select aria-label="Tipo de transporte" value={filter} onChange={e=>setFilter(e.target.value)}>{['TODOS','IPC','WEB','REST','SOAP','WEBHOOK'].map(x=><option key={x} value={x}>{x}</option>)}</select><button onClick={refresh}>Atualizar</button><button onClick={exportText}>Exportar TXT</button><button onClick={()=>{if(window.confirm('Apagar o histórico de debug?'))void api('debug:clear').then(refresh)}}>Limpar</button></nav>
 <div className="debugScreen" role="log" aria-live="off">{shown.length===0?<p>Sem registros {filter==='TODOS'?'':'para '+filter}. Nenhum tráfego externo é simulado.</p>:shown.map(r=><div key={r.id}><span>{r.at}</span> <b>[{r.category}]</b> {r.method} {r.endpoint} <em>{r.status}</em> {r.duration_ms}ms</div>)}</div>
 <footer>Máximo 1.500 eventos · Metadados locais, sem payloads, tokens ou senhas · Captura iniciada nesta versão. Requisições realizadas fora deste aplicativo não são capturadas.</footer>{error&&<p>{error}</p>}</section></div>
}
