import React,{useEffect,useState} from 'react';
import {A1E2Proof} from './A1E2Proof';
import {open as openDialog} from '@tauri-apps/plugin-dialog';
type Obj=Record<string,any>;
export function FiscalWorkspace({api,view='all'}:{api:(channel:string,...args:any[])=>Promise<any>;view?:'all'|'partners'|'documents'}){
 const [partners,setPartners]=useState<Obj[]>([]),[docs,setDocs]=useState<Obj[]>([]),[events,setEvents]=useState<Obj[]>([]);
 const [partner,setPartner]=useState<Obj>({kind:'fornecedor',name:'',document:'',uf:'RJ',ie:'',address:''});
 const [draft,setDraft]=useState<Obj>({direction:'entrada',partner_id:'',sale_id:'',operation_description:'',cfop:''});
 const [selected,setSelected]=useState<number|null>(null);
 const [notice,setNotice]=useState('');const [busy,setBusy]=useState(false);
 const reload=async()=>{const [ps,ds]=await Promise.all([api('fiscal:partners:list'),api('fiscal:documents:list')]);setPartners(ps);setDocs(ds)};
 useEffect(()=>{void reload().catch(e=>setNotice(String(e)))},[]);
 const run=async(fn:()=>Promise<any>)=>{setBusy(true);try{await fn();await reload();setNotice('Dados gravados. Nenhum documento foi enviado à SEFAZ.')}catch(e){setNotice(String(e))}finally{setBusy(false)}};
 return <section className="fiscalWorkspace">
  <A1E2Proof api={api}/>
  {(view==='all'||view==='partners')&&<div><h3>Etapa B · Fornecedores e clientes fiscais</h3>
  <p>Cadastro separado dos clientes comerciais. CPF/CNPJ é validado quando informado; consulte o contador para documentos e exceções fiscais.</p>
  <div className="formGrid">
   <label>Tipo<select value={partner.kind} onChange={e=>setPartner({...partner,kind:e.target.value})}><option value="fornecedor">Fornecedor</option><option value="cliente">Cliente</option></select></label>
   <label>Nome / Razão social<input value={partner.name} onChange={e=>setPartner({...partner,name:e.target.value})}/></label>
   <label>CPF/CNPJ (sem pontuação)<input value={partner.document} onChange={e=>setPartner({...partner,document:e.target.value.toUpperCase().replace(/[^A-Z0-9]/g,'').slice(0,14)})}/></label>
   <label>UF<input value={partner.uf} maxLength={2} onChange={e=>setPartner({...partner,uf:e.target.value.toUpperCase()})}/></label>
   <label>Inscrição estadual<input value={partner.ie} onChange={e=>setPartner({...partner,ie:e.target.value})}/></label>
   <label>Endereço<input value={partner.address} onChange={e=>setPartner({...partner,address:e.target.value})}/></label>
  </div>
  <div className="actionRow"><button className="primary" disabled={busy} onClick={()=>void run(()=>api('fiscal:partner:save',partner))}>Salvar parceiro</button><button onClick={()=>setPartner({kind:'fornecedor',name:'',document:'',uf:'RJ',ie:'',address:''})}>Novo cadastro</button></div>
  <div className="tableScroll"><table><thead><tr><th>Tipo</th><th>Nome</th><th>Documento</th><th>UF</th><th></th></tr></thead><tbody>{partners.map(p=><tr key={p.id}><td>{p.kind}</td><td>{p.name}</td><td>{p.document?'••••'+String(p.document).slice(-4):'Não informado'}</td><td>{p.uf}</td><td><button onClick={()=>setPartner(p)}>Editar</button></td></tr>)}</tbody></table></div>
  </div>}{(view==='all'||view==='documents')&&<div><h3>Etapas C / D · Controle de documentos</h3>
  <p>Rascunhos vinculáveis a uma operação comercial. XML recebido pode ser referenciado e ter SHA-256 calculado, sem validação fiscal. Assinatura, transmissão, autorização, cancelamento e DANFE oficial ainda exigem integração homologada.</p>
  <div className="formGrid">
   <label>Operação<select value={draft.direction} onChange={e=>setDraft({...draft,direction:e.target.value,partner_id:''})}><option value="entrada">NF-e entrada / compra</option><option value="saida">NF-e saída / venda</option></select></label>
   <label>Parceiro<select value={draft.partner_id} onChange={e=>setDraft({...draft,partner_id:e.target.value})}><option value="">Selecionar</option>{partners.filter(p=>p.kind===(draft.direction==='entrada'?'fornecedor':'cliente')).map(p=><option key={p.id} value={p.id}>{p.name}</option>)}</select></label>
   <label>ID da operação no caixa (opcional)<input inputMode="numeric" value={draft.sale_id} onChange={e=>setDraft({...draft,sale_id:e.target.value.replace(/\D/g,'')})}/></label>
   <label>CFOP (preenchimento após revisão)<input maxLength={4} value={draft.cfop} onChange={e=>setDraft({...draft,cfop:e.target.value.replace(/\D/g,'')})}/></label>
   <label>Descrição da operação<input value={draft.operation_description} onChange={e=>setDraft({...draft,operation_description:e.target.value})}/></label>
  </div>
  <button className="primary" disabled={busy||!draft.partner_id} onClick={()=>void run(()=>api('fiscal:document:create',{...draft,partner_id:Number(draft.partner_id),sale_id:draft.sale_id?Number(draft.sale_id):null}))}>Criar rascunho fiscal</button>
  <div className="tableScroll"><table><thead><tr><th>Nº interno</th><th>Tipo</th><th>Parceiro</th><th>Status</th><th>Ações</th></tr></thead><tbody>{docs.map(d=><tr key={d.id}><td>#{d.id}</td><td>{d.direction}</td><td>{d.partner_name}</td><td>{d.status}</td><td><div className="actionRow"><button onClick={()=>{setSelected(d.id);void api('fiscal:document:events',{id:d.id}).then(setEvents)}}>Histórico</button><button onClick={()=>void api('fiscal:document:preflight',{id:d.id}).then((res:Obj)=>setNotice('Diagnóstico local: '+(res.bloqueios||[]).join(' | '))).catch(e=>setNotice(String(e)))}>Diagnóstico pré-envio</button><button disabled={busy||!['rascunho','pendente_configuracao','pendente_revisao','xml_importado'].includes(d.status)} onClick={async()=>{try{const path=await openDialog({multiple:false,directory:false,filters:[{name:'XML NF-e recebido',extensions:['xml']}]});if(typeof path==='string')await run(()=>api('fiscal:document:import_xml',{id:d.id,path}))}catch(e){setNotice(String(e))}}}>Referenciar XML</button></div></td></tr>)}</tbody></table></div>
  {selected!==null&&<div className="fiscalNotice"><strong>Histórico do documento #{selected}</strong><ul>{events.map(e=><li key={e.id}>{e.at} — {e.event}: {e.note}</li>)}</ul></div>}
  </div>}
  {notice&&<p role="status" className="fiscalNotice">{notice}</p>}
  <p className="fiscalNotice">⚠️ Sem emissão real: nenhuma NF-e pode receber status "autorizado" por ação manual. A receita, custos e pagamentos do Caixa continuam independentes do status fiscal.</p>
 </section>
}
