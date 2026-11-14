import React,{useCallback,useEffect,useState} from 'react';
type Api=(channel:string,...args:any[])=>Promise<any>;
type Batch={id:number;date:string;origin:string;status:string;count:number;approved_by?:string|null;transmitted:false};
const localDate=()=>{const d=new Date();const y=d.getFullYear(),m=String(d.getMonth()+1).padStart(2,'0'),day=String(d.getDate()).padStart(2,'0');return `${y}-${m}-${day}`};
export function FiscalBatchPanel({api}:{api:Api}){
 const [date,setDate]=useState(localDate),[enabled,setEnabled]=useState(false),[hour,setHour]=useState('18:00');
 const [rows,setRows]=useState<Batch[]>([]),[busy,setBusy]=useState(false),[notice,setNotice]=useState('');
 const refresh=useCallback(async()=>{const [settings,lots]=await Promise.all([api('batch:settings'),api('batch:list')]);setEnabled(settings.enabled);setHour(settings.time);setRows(lots)},[api]);
 useEffect(()=>{void refresh().catch(e=>setNotice(String(e)))},[refresh]);
 // Apenas agenda preparação de lista de revisão enquanto a aplicação está aberta.
 // Nunca aciona SEFAZ; após fechamento do aplicativo não há serviço em segundo plano.
 useEffect(()=>{let live=true;const tick=()=>{void api('batch:tick').then(v=>{if(live&&v.created){setNotice('Lote agendado para revisão. Nenhum documento foi enviado.');void refresh()}}).catch(e=>{if(live)setNotice(String(e))})};tick();const timer=setInterval(tick,60000);return()=>{live=false;clearInterval(timer)}},[api,refresh]);
 const run=async(fn:()=>Promise<unknown>,ok:string)=>{if(busy)return;setBusy(true);setNotice('');try{await fn();await refresh();setNotice(ok)}catch(e){setNotice(String(e))}finally{setBusy(false)}};
 return <section className="panel">
  <h3>Gestão de lotes NF-e — preparação e aprovação</h3>
  <p className="fiscalNotice">SEM ENVIO OFICIAL: as notas são somente rascunhos. O agendamento monta uma lista de revisão com o aplicativo aberto. Aprovar um lote NÃO valida XML, não assina e não transmite à SEFAZ.</p>
  <div className="formGrid"><label>Data dos rascunhos<input type="date" value={date} onChange={e=>setDate(e.target.value)}/></label>
  <label>Horário local de preparação<input type="time" value={hour} onChange={e=>setHour(e.target.value)}/></label>
  <label><input type="checkbox" checked={enabled} onChange={e=>setEnabled(e.target.checked)}/> Preparar automaticamente para revisão (aplicativo aberto)</label></div>
  <div className="actionRow"><button disabled={busy} onClick={()=>void run(()=>api('batch:settings:save',{enabled,time:hour}),'Agendamento salvo. Nenhuma transmissão configurada.')}>Salvar agendamento</button>
  <button className="primary" disabled={busy} onClick={()=>void run(()=>api('batch:create',{date}),'Lote manual criado para revisão. Nenhum envio foi executado.')}>Preparar lote manual</button>
  <button disabled={busy} onClick={()=>void run(()=>api('batch:tick'),'Verificação de horário concluída; sem envio.')}>Verificar horário</button></div>
  <div className="tableScroll"><table><thead><tr><th>Lote</th><th>Data</th><th>Origem</th><th>Rascunhos</th><th>Estado administrativo</th><th>Ação</th></tr></thead><tbody>{rows.map(b=><tr key={b.id}>
   <td>#{b.id}</td><td>{b.date}</td><td>{b.origin==='scheduled'?'Programado':'Manual'}</td><td>{b.count}</td><td>{b.status==='approved'?'Aprovado para preparação':'Revisão pendente'}</td>
   <td>{b.status==='review'?<button disabled={busy} onClick={()=>{if(window.confirm(`Confirmar APROVAÇÃO ADMINISTRATIVA do lote #${b.id}? NÃO será enviado à SEFAZ.`)){void run(()=>api('batch:approve',{id:b.id,confirmed:true}),'Lote aprovado administrativamente, SEM transmissão SEFAZ.')}}}>Revisar e aprovar</button>:<span>Não transmitido</span>}</td>
   </tr>)}</tbody></table></div>
   <p role="status" className="fiscalNotice">{notice||'Nenhum lote será marcado como autorizado automaticamente.'}</p>
 </section>
}
