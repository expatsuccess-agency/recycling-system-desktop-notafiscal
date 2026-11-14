import React,{useState} from 'react';
type Api=(channel:string,...args:any[])=>Promise<any>;
type Kind='caixa'|'compras'|'vendas'|'fiscal'|'consolidado';
const day=()=>{const d=new Date();return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`};
export function ReportsPanel({api}:{api:Api}){
 const [from,setFrom]=useState(day),[to,setTo]=useState(day),[busy,setBusy]=useState(false),[notice,setNotice]=useState('');
 const generate=async(kind:Kind)=>{setBusy(true);setNotice('');try{const v=await api('reports:generate',{kind,from,to});setNotice(`PDF criado: ${v.path} (${v.pages} página(s)). Nenhum documento fiscal foi emitido.`)}catch(e){setNotice(String(e))}finally{setBusy(false)}};
 return <section className="panel"><h3>Relatórios PDF — V8.3</h3><p>Arquivos operacionais em Documentos/Recycling System/Relatorios. Não são DANFE, NF-e ou substitutos dos XML autorizados.</p>
  <div className="formGrid"><label>Data inicial<input type="date" value={from} onChange={e=>setFrom(e.target.value)}/></label><label>Data final<input type="date" value={to} onChange={e=>setTo(e.target.value)}/></label></div>
  <div className="actionRow">{([['caixa','Caixas'],['compras','Compras'],['vendas','Vendas'],['fiscal','NF-e (controle interno)'],['consolidado','Consolidado'] ] as [Kind,string][]).map(([kind,label])=><button key={kind} type="button" disabled={busy||from>to} onClick={()=>void generate(kind)}>Gerar PDF · {label}</button>)}</div>
  <p className="fiscalNotice">Ao fechar um Caixa, o PDF consolidado correspondente é gerado automaticamente. Falhas na gravação do arquivo não anulam o fechamento.</p>
  {notice&&<p role="status" className="fiscalNotice">{notice}</p>}
 </section>
}
