import React,{useEffect,useState} from 'react';
import {open as openDialog} from '@tauri-apps/plugin-dialog';
import {FiscalWorkspace} from './FiscalWorkspace';
import {A1E2Proof} from './A1E2Proof';

type Obj=Record<string,any>;
type Api=(channel:string,...args:any[])=>Promise<any>;
type FiscalTab='empresa'|'certificado'|'parceiros'|'documentos'|'monitor';
const tabs:{id:FiscalTab;label:string}[]=[
 {id:'empresa',label:'Dados da Empresa'},
 {id:'certificado',label:'Certificado Digital'},
 {id:'parceiros',label:'Parceiros'},
 {id:'documentos',label:'Documentos Fiscais'},
 {id:'monitor',label:'Monitor de Transmissões'}
];
const companyFields:[string,string][]=[
 ['razao_social','Razão social'],['nome_fantasia','Nome fantasia'],['cnpj','CNPJ'],
 ['inscricao_estadual','Inscrição Estadual'],['uf','UF'],['codigo_municipio','Código IBGE'],
 ['municipio','Município'],['cep','CEP'],['logradouro','Logradouro'],
 ['numero','Número'],['bairro','Bairro'],['complemento','Complemento']
];
export function FiscalPortal({api}:{api:Api}){
 const [tab,setTab]=useState<FiscalTab>('empresa');
 const [company,setCompany]=useState<Obj>({uf:'RJ',crt:'',ambiente:'homologacao'});
 const [pending,setPending]=useState<Obj|null>(null);
 const [events,setEvents]=useState<Obj[]>([]);
 const [password,setPassword]=useState('');
 const [notice,setNotice]=useState('');
 const [busy,setBusy]=useState(false);
 const refreshCompany=()=>api('fiscal:company:get').then(v=>setCompany(v));
 const refreshPending=()=>api('fiscal:pendencias').then(setPending);
 const refreshEvents=()=>api('debug:events').then(setEvents);
 useEffect(()=>{void refreshCompany().catch(e=>setNotice(String(e)))},[]);
 useEffect(()=>{if(tab==='monitor'){void refreshEvents().catch(e=>setNotice(String(e)))}},[tab]);
 const execute=async(fn:()=>Promise<unknown>,ok:string)=>{
  if(busy)return;setBusy(true);setNotice('');
  try{await fn();setNotice(ok)}catch(e){setNotice(String(e))}finally{setBusy(false);setPassword('')}
 };
 return <section className="panel fiscalPanel fiscalPortal">
  <h2>Dados Fiscais · NF-e / NFC-e</h2>
  <p>RJ · Configuração em homologação. Nenhuma NF-e é enviada ou autorizada por esta versão.</p>
  <nav className="fiscalTabs" aria-label="Módulos fiscais" role="tablist">
   {tabs.map(t=><button type="button" key={t.id} role="tab" aria-selected={tab===t.id} className={tab===t.id?'active':''} onClick={()=>{setTab(t.id);setNotice('')}}>{t.label}</button>)}
  </nav>
  {tab==='empresa'&&<div role="tabpanel">
   <h3>Dados da Empresa</h3>
   <div className="formGrid">
    {companyFields.map(([key,label])=><label key={key}>{label}<input value={company[key]||''} onChange={e=>setCompany({...company,[key]:key==='uf'?e.target.value.toUpperCase():key==='cnpj'?e.target.value.toUpperCase().replace(/[^A-Z0-9]/g,'').slice(0,14):key==='cep'||key==='codigo_municipio'?e.target.value.replace(/\D/g,''):e.target.value})}/></label>)}
    <label>Regime tributário (CRT)<select value={company.crt??''} onChange={e=>setCompany({...company,crt:e.target.value})}><option value="">A definir — pendente de validação</option><option value="1">1 · Simples Nacional</option><option value="2">2 · Simples Nacional — excesso</option><option value="3">3 · Regime Normal</option><option value="4">4 · MEI — verificar aplicabilidade</option></select></label>
    <label>Ambiente fiscal<select value={company.ambiente||'homologacao'} onChange={e=>setCompany({...company,ambiente:e.target.value})}><option value="homologacao">Homologação (testes)</option><option value="producao" disabled>Produção (bloqueada)</option></select></label>
   </div>
   <div className="actionRow">
    <button className="primary" disabled={busy} onClick={()=>void execute(async()=>{await api('fiscal:company:save',company);await refreshPending()},'Dados salvos em modo rascunho. Emissão bloqueada.')}>Salvar dados fiscais</button>
    <button disabled={busy} onClick={()=>void execute(refreshPending,'Diagnóstico atualizado.')}>Verificar pendências fiscais</button>
   </div>
   {pending&&<div className="fiscalNotice"><h4>Pendências fiscais · emissão bloqueada</h4><ul>{(pending.pendencias||[]).map((v:string,i:number)=><li key={i}>{v}</li>)}</ul></div>}
  </div>}
  {tab==='certificado'&&<div role="tabpanel">
   <h3>Certificado Digital A1</h3>
   <p>O certificado A1 permanece fora do SQLite. Nenhuma senha é armazenada. Não inclui validação da cadeia ICP-Brasil ou revogação.</p>
   <label>Caminho do certificado .pfx/.p12<input readOnly value={company.certificado_a1_caminho||''} placeholder="Nenhum certificado selecionado"/></label>
   <div className="actionRow">
    <button type="button" onClick={async()=>{try{const p=await openDialog({multiple:false,directory:false,filters:[{name:'Certificado A1',extensions:['pfx','p12']}]});if(typeof p==='string')setCompany({...company,certificado_a1_caminho:p})}catch(e){setNotice(String(e))}}}>Selecionar A1</button>
    <button type="button" onClick={()=>setCompany({...company,certificado_a1_caminho:''})}>Limpar seleção</button>
    <button className="primary" disabled={busy} onClick={()=>void execute(async()=>{await api('fiscal:company:save',company);await refreshCompany()},'Referência do certificado salva; senha não gravada.')}>Salvar referência A1</button>
   </div>
   <div className="fiscalNotice"><label>Senha do certificado (não armazenada)<input type="password" autoComplete="off" value={password} onChange={e=>setPassword(e.target.value)}/></label>
    <div className="actionRow"><button disabled={busy||!company.certificado_a1_caminho||!password} onClick={()=>void execute(async()=>{const result=await api('fiscal:a1:inspect',{senha:password});setNotice(`E1: ${JSON.stringify(result)}`)},'Inspeção local concluída.')}>Verificar certificado A1 (E1)</button></div>
   </div>
   <A1E2Proof api={api}/>
   <p className="fiscalNotice">E1 e E2 comprovam apenas verificações locais; não são assinatura XML NF-e nem autorização SEFAZ.</p>
  </div>}
  {tab==='parceiros'&&<div role="tabpanel"><FiscalWorkspace api={api} view="partners"/></div>}
  {tab==='documentos'&&<div role="tabpanel">
   <FiscalWorkspace api={api} view="documents"/>
   <div className="fiscalNotice"><strong>Transmissão NF-e — somente ação manual</strong>
    <p>Sequência futura: Revisar → Validar XSD → Assinar → Transmitir NF-e → Consultar protocolo. Nenhuma etapa é executada automaticamente neste módulo.</p>
    <button type="button" disabled title="Conector SEFAZ, XML NF-e oficial e assinatura ainda não homologados">Transmitir NF-e (indisponível)</button>
    <p>Um agendador futuro poderá retomar consultas de requisições já enviadas, mas nunca autorizar uma NF-e por conta própria.</p>
   </div>
  </div>}
  {tab==='monitor'&&<div role="tabpanel">
   <h3>Monitor de Transmissões</h3>
   <p>Histórico de eventos realmente registrados pelo aplicativo; sem mensagens simuladas ou XML completo.</p>
   <div className="actionRow"><button onClick={()=>void execute(refreshEvents,'Eventos atualizados.')}>Atualizar histórico</button><button disabled>Retomar pendências (não implementado)</button></div>
   <div className="tableScroll"><table><thead><tr><th>Data</th><th>Canal</th><th>Método</th><th>Status</th><th>Duração</th></tr></thead><tbody>{events.length===0?<tr><td colSpan={5}>Nenhuma transmissão registrada.</td></tr>:events.map(e=><tr key={e.id}><td>{e.at}</td><td>{e.category}</td><td>{e.method}</td><td>{e.status}</td><td>{e.duration_ms} ms</td></tr>)}</tbody></table></div>
   <p className="fiscalNotice">O monitor não intercepta tráfego de terceiros. SOAP e REST aparecerão somente quando transportes reais forem instrumentados.</p>
  </div>}
  {notice&&<p className="fiscalNotice" role="status">{notice}</p>}
 </section>
}
