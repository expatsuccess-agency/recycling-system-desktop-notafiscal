import {open as openDialog} from '@tauri-apps/plugin-dialog';
import {listen} from '@tauri-apps/api/event';
import React,{useEffect,useState,useCallback,useRef} from 'react';
import {createRoot} from 'react-dom/client';
import {lxo,setSessionToken} from './api/lxo';
import {Scale,ShoppingBasket,ShoppingCart,Wallet,Users,Settings,BarChart3,LogOut,Trash2,FileText,Check,Search,ArrowUpRight,ArrowDownLeft,ChevronRight,RefreshCw,X,Eye,EyeOff,ChevronDown,UserRound,LockKeyhole,LoaderCircle,Grid2X2} from 'lucide-react';
import './style.css';
type Obj=Record<string,any>;
type CaixaSession={id:number;status:'open'|'closed';openedAt:string;closedAt?:string;initialBalance:number;totalSales:number;totalPurchases:number;totalWithdrawals:number;totalDeposits:number;finalBalance:number;cashSales:number;grossProfit:number|null;historyLogs:{user:string;action:string;timestamp:string}[];timeline?:TimelineEvent[]};
const api=(channel:string,...args:any[]):Promise<any>=>lxo.invoke(channel,...args);
const brl=(c:number)=>new Intl.NumberFormat('pt-BR',{style:'currency',currency:'BRL'}).format((Number(c)||0)/100);
const digits=(v:string)=>{const raw=v.trim().replace(/\s/g,'');const clean=raw.includes(',')?raw.replace(/\./g,'').replace(',','.'):raw;return Math.round(Number(clean||'0')*100)};
const cashInput=(v:string)=>v.replace(/[^\d,.]/g,'').replace(/(,.*),/g,'$1');
const cashDisplay=(v:string)=>{const n=digits(v);return Number.isFinite(n)?(n/100).toLocaleString('pt-BR',{minimumFractionDigits:2,maximumFractionDigits:2}):'0,00'};
const date=(d?:string)=>d?new Date(d).toLocaleString('pt-BR'):'—';
// Exibe até três casas decimais, sem zeros desnecessários (ex.: 45 kg; 45,32 kg).
const formatKg=(grams:number)=>`${new Intl.NumberFormat('pt-BR',{minimumFractionDigits:0,maximumFractionDigits:3}).format((Number(grams)||0)/1000)} kg`;
const ptErrors:Record<string,string>={'NCM_OBRIGATORIO_OITO_DIGITOS':'Informe o NCM do produto com 8 dígitos. Confira o código com a contabilidade.','CERTIFICADO_A1_ARQUIVO_INVALIDO':'Selecione um arquivo A1 .pfx ou .p12 existente.','CST_ICMS_INVALIDO':'O CST de ICMS deve conter 2 dígitos.','CSOSN_INVALIDO':'O CSOSN deve conter 3 dígitos.','CFOP_INVALIDO':'O CFOP deve conter 4 dígitos.','CEST_INVALIDO':'O CEST deve conter 7 dígitos.',INVALID_CLIENT:'Preencha os dados obrigatórios do cliente.',INVALID_SCALE:'Verifique os dados cadastrados.',INVALID_AMOUNT:'Informe um valor válido.',INVALID_SALE:'Não foi possível registrar a operação.',NO_OPEN_CAIXA:'Abra um caixa antes de continuar.', 'CREDENCIAIS_INVÁLIDAS':'Usuário ou senha incorretos. Confira os dados e tente novamente.', 'ACESSO_NEGADO':'Você não tem permissão para realizar esta operação.', 'ALTERAÇÃO_DE_SENHA_OBRIGATÓRIA':'Antes de continuar, cadastre uma nova senha.', 'LICENÇA_INVÁLIDA':'A licença precisa ser verificada ou ativada.', 'CONFIRMAÇÃO_OBRIGATÓRIA':'Digite a frase de confirmação para continuar.', 'BANCO_SEM_CAMINHO':'Não foi possível localizar o banco de dados.', 'LOCK_FAILED':'O banco de dados está ocupado. Tente novamente.', 'INVALID_DATE':'Confira as datas informadas.', 'INVALID_PASSWORD':'Confira a senha informada.', 'DADOS_DE_USUÁRIO_INVÁLIDOS':'Informe um usuário válido e uma senha de 8 números.', 'ERRO_BACKUP':'Não foi possível criar a cópia de segurança do banco. Nenhum dado foi apagado.', 'ERRO_LIMPEZA':'A limpeza falhou. O banco foi protegido pela transação.', 'ERRO_CONFIRMACAO':'A gravação da limpeza falhou. Confira o diagnóstico no registro.'};
const errorPt=(error:any)=>{const raw=String(error?.message||error||'');const code=Object.keys(ptErrors).find(k=>raw.includes(k));if(code)return ptErrors[code];const cleaned=raw.replace(/^Error invoking remote method[^:]*:\s*/i,'').replace(/^Error:\s*/,'').trim();if(/^[A-ZÀ-Ü_]+$/.test(cleaned))return 'Não foi possível concluir a operação. Verifique os dados e tente novamente.';return cleaned||'Não foi possível concluir a operação.'};
const today=()=>new Date().toLocaleDateString('en-CA');
const durationBetween=(a:string,b:string)=>{const ms=Date.parse(b)-Date.parse(a);if(!Number.isFinite(ms)||ms<0)return '—';const m=Math.floor(ms/60000);return m<60?`${m} min`:m<1440?`${Math.floor(m/60)} h ${m%60} min`:`${Math.floor(m/1440)} d ${Math.floor((m%1440)/60)} h`;};
const icons=['♻️','🔩','🧱','📦','🥫','🪵','🔧','🛢️','⚖️','🔌','🔋','🖥️','💻','📱','🧲','🔗','🧴','🧃','🍶','🫙','🧵','👕','👟','🪑','🪟','🚲','🚗','🛞','🧰','🪣','🧯','🗞️','📚','📄','🥤','🍾','🪙','⚙️'];
const iconGroups:Record<string,string[]>={
 'Reciclagem':['♻️','⚙️','⚖️'],
 'Metais':['🔩','🥫','🔧','🛢️','🧲','🔗','🪙'],
 'Papel':['🧱','📦','🗞️','📚','📄'],
 'Plásticos':['🧴','🧃','🥤','🪣'],
 'Vidro':['🍾','🍶','🫙'],
 'Outros':['🪵','🔌','🔋','🖥️','💻','📱','🧵','👕','👟','🪑','🪟','🚲','🚗','🛞','🧰','🧯']
};
function MaterialIconPicker({value,onChange}:{value:string;onChange:(icon:string)=>void}){
 const [open,setOpen]=useState(false),[filter,setFilter]=useState(''),[group,setGroup]=useState('Todos');
 const ref=useRef<HTMLDivElement|null>(null);
 useEffect(()=>{
  if(!open)return;
  const outside=(e:PointerEvent)=>{if(ref.current&&!ref.current.contains(e.target as Node))setOpen(false)};
  const escape=(e:KeyboardEvent)=>{if(e.key==='Escape')setOpen(false)};
  document.addEventListener('pointerdown',outside);document.addEventListener('keydown',escape);
  return()=>{document.removeEventListener('pointerdown',outside);document.removeEventListener('keydown',escape)};
 },[open]);
 const categoryLabels=Object.keys(iconGroups);
 const all=Array.from(new Set([...icons,value]));
 const filtered=all.filter(ic=>{
   const groupName=categoryLabels.find(g=>iconGroups[g].includes(ic))||'Outros';
   return (group==='Todos'||group===groupName) && (!filter||`${ic} ${groupName}`.toLocaleLowerCase('pt-BR').includes(filter.toLocaleLowerCase('pt-BR')));
 });
 return <div className="iconPicker" ref={ref}>
  <button type="button" className="iconPickerTrigger" aria-label="Escolher ícone do material" aria-haspopup="dialog" aria-expanded={open} onClick={()=>setOpen(v=>!v)}><span className="iconPickerSelected">{value||'♻️'}</span><ChevronDown size={19}/></button>
  {open&&<div className="iconPickerPanel" role="dialog" aria-label="Escolha o ícone do material">
   <header><Grid2X2 size={20}/><strong>Escolha o ícone do material</strong><button type="button" aria-label="Fechar seletor" className="iconPickerClose" onClick={()=>setOpen(false)}><X size={18}/></button></header>
   <div className="iconPickerSearch"><Search size={18}/><input aria-label="Buscar ícone" value={filter} placeholder="Buscar ícone ou categoria..." onChange={e=>setFilter(e.target.value)}/></div>
   <div className="iconPickerCategories">{['Todos',...categoryLabels].map(g=><button type="button" key={g} className={group===g?'current':''} onClick={()=>setGroup(g)}>{g}</button>)}</div>
   <div className="iconPickerGrid">{filtered.map(ic=><button type="button" key={ic} aria-label={`Ícone ${ic}`} aria-pressed={value===ic} className={value===ic?'current':''} onClick={()=>{onChange(ic);setOpen(false);setFilter('')}}><span>{ic}</span>{value===ic&&<Check size={14} className="iconPickerCheck"/>}</button>)}{filtered.length===0&&<p>Nenhum ícone encontrado.</p>}</div>
   <small>Selecione um ícone para identificar o material.</small>
  </div>}
 </div>
}
const methods=[{code:'pix',label:'PIX',icon:'◆'},{code:'cash',label:'Dinheiro',icon:'R$'},{code:'card',label:'Cartão',icon:'▣'},{code:'transfer',label:'Transferência',icon:'⇄'}];
function App({identity,onLogout}:{identity:Obj;onLogout:()=>void}){
 const isAdmin=identity.role==='admin';
 const [tab,setTab]=useState<'weigh'|'vendas'|'caixa'|'clients'|'config'|'finance'>('weigh');
 const [configTab,setConfigTab]=useState<'products'|'users'|'control'|'fiscal'|'maintenance'>('products');
 const [caixaTab,setCaixaTab]=useState<'current'|'previous'>('current');
 const [articles,setArticles]=useState<Obj[]>([]),[clients,setClients]=useState<Obj[]>([]),[categories,setCategories]=useState<Obj[]>([]);
 const [weight,setWeight]=useState(''),[productId,setProductId]=useState(''),[clientId,setClientId]=useState('');
 const [buyPrice,setBuyPrice]=useState('');const [newUser,setNewUser]=useState({username:'',password:'',role:'employee'});const [users,setUsers]=useState<Obj[]>([]);const [resetPwd,setResetPwd]=useState(''),[resetUser,setResetUser]=useState('');
 const [financial,setFinancial]=useState<Obj|null>(null),[expenseCategories,setExpenseCategories]=useState<Obj[]>([]),[expenseCategoryName,setExpenseCategoryName]=useState('');
 const [expense,setExpense]=useState({categoryId:'',description:'',value:'',date:today()}),[reportPeriod,setReportPeriod]=useState('day');
 const [saleProductId,setSaleProductId]=useState(''),[saleWeight,setSaleWeight]=useState(''),[salePrice,setSalePrice]=useState('');
 const [saleCart,setSaleCart]=useState<Obj[]>([]),[salesHistory,setSalesHistory]=useState<Obj[]>([]),[saleHistoryDetail,setSaleHistoryDetail]=useState<Obj|null>(null);
 const [buyerCompany,setBuyerCompany]=useState('');
 const [salesFrom,setSalesFrom]=useState(today().slice(0,8)+'01'),[salesTo,setSalesTo]=useState(today());
 const [cart,setCart]=useState<Obj[]>([]),[paymentOpen,setPaymentOpen]=useState(false),[method,setMethod]=useState('cash');
 const [caixa,setCaixa]=useState<CaixaSession|null>(null),[previous,setPrevious]=useState<CaixaSession[]>([]);
 const [dialog,setDialog]=useState<null|'open'|'deposit'|'withdrawal'|'close'|'period'>(null);
 const [amount,setAmount]=useState(''),[caixaNumber,setCaixaNumber]=useState(''),[busy,setBusy]=useState(false),[message,setMessage]=useState(''),[error,setError]=useState('');
 const [clientForm,setClientForm]=useState<Obj>({gender:'',first_name:'',last_name:'',document:''});
 const [fiscalCompany,setFiscalCompany]=useState<Obj>({razao_social:'',nome_fantasia:'',cnpj:'',inscricao_estadual:'',crt:'1',uf:'',codigo_municipio:'',municipio:'',cep:'',logradouro:'',numero:'',bairro:'',complemento:'',ambiente:'homologacao',certificado_a1_caminho:''});
 const [catName,setCatName]=useState(''),[productForm,setProductForm]=useState<Obj>({code_article:'',designation:'',price_per_kg:0,category_id:null,icon:'♻️',price_editable:false,cost_per_kg:null,sale_price_per_kg:null,ncm:'',cest:'',origem_icms:'0',cst_icms:'',csosn:'',cfop_entrada:'',cfop_saida:''});
 const [from,setFrom]=useState(today().slice(0,8)+'01'),[to,setTo]=useState(today()),[search,setSearch]=useState('');
 const [previousSearch,setPreviousSearch]=useState(''),[periodFrom,setPeriodFrom]=useState(today().slice(0,8)+'01'),[periodTo,setPeriodTo]=useState(today()),[periodResult,setPeriodResult]=useState<Obj|null>(null);
 const productFocus=useRef<HTMLInputElement|null>(null);
 const weightFocus=useRef<HTMLInputElement|null>(null);
 const buyPriceFocus=useRef<HTMLInputElement|null>(null);
 const [cashDetail,setCashDetail]=useState<Obj|null>(null);
 const [resetBusy,setResetBusy]=useState(false);
 const [resetPhrase,setResetPhrase]=useState('');
 const [resetConfirmed,setResetConfirmed]=useState(false);
 const [resetLogs,setResetLogs]=useState<{time:string;level:'info'|'ok'|'error';message:string}[]>([]);
 const addResetLog=(message:string,level:'info'|'ok'|'error'='info')=>setResetLogs(prev=>[...prev,{time:new Date().toLocaleTimeString('pt-BR'),level,message}]);
 useEffect(()=>{let removed=false;let off:(()=>void)|undefined;void listen<{message:string;level:'info'|'ok'|'error'}>('maintenance:progress',event=>{if(!removed&&event.payload?.message)addResetLog(event.payload.message,event.payload.level||'info')}).then(unlisten=>{if(removed)unlisten();else off=unlisten}).catch(()=>{});return()=>{removed=true;off?.()}},[]);
 const paymentCashFocus=useRef<HTMLButtonElement|null>(null);
 const [rows,setRows]=useState<Obj[]>([]),[detail,setDetail]=useState<Obj|null>(null),[expanded,setExpanded]=useState<number|null>(null);
 const notify=(s:string)=>{setMessage(s);setError('')};const fail=(e:any)=>{setError(errorPt(e));setMessage('')};
 // Avisos expiram após sete segundos e desaparecem ao mudar de página.
 useEffect(()=>{if(!message&&!error)return;const timer=window.setTimeout(()=>{setMessage('');setError('')},7000);return()=>window.clearTimeout(timer)},[message,error]);
 useEffect(()=>{setMessage('');setError('')},[tab,caixaTab,configTab]);
 const refresh=useCallback(async()=>{
  const [p,c,k,active]=await Promise.all([api('articles:all'),isAdmin?api('clients:list'):Promise.resolve([]),isAdmin?api('categories:list'):Promise.resolve([]),api('caixa:current')]);
  setArticles(p);setClients(c);setCategories(k);setCaixa(active);
 },[isAdmin]);
 const loadPrevious=useCallback(async()=>{try{setPrevious(await api('caixa:list',{from:periodFrom,to:periodTo,search:previousSearch}))}catch(e){fail(e)}},[periodFrom,periodTo,previousSearch]);
 const loadHistory=useCallback(async()=>{try{setRows(await api('history',from,to,'all'))}catch(e){fail(e)}},[from,to]);
 useEffect(()=>{refresh().catch(fail)},[refresh]);
 useEffect(()=>{if(tab==='weigh'&&!dialog&&!paymentOpen)window.requestAnimationFrame(()=>productFocus.current?.focus())},[tab,dialog,paymentOpen]);
 useEffect(()=>{if(tab==='caixa'&&caixaTab==='previous')loadPrevious()},[tab,caixaTab,loadPrevious]);
 useEffect(()=>{if(tab==='config'&&configTab==='control')loadHistory()},[tab,configTab,loadHistory]);
 useEffect(()=>{if(tab==='config'&&isAdmin)api('auth:users').then(setUsers).catch(fail)},[tab,isAdmin]);
 const loadFinance=useCallback(async()=>{if(!isAdmin)return;try{setFinancial(await api('finance:report',{from,to,groupBy:reportPeriod}));setExpenseCategories(await api('finance:categories'))}catch(e){fail(e)}},[isAdmin,from,to,reportPeriod]);
 useEffect(()=>{if(tab==='finance')void loadFinance()},[tab,loadFinance]);
 const selected=articles.find(p=>String(p.id)===productId);
 const kg=Number(weight.replace(',','.'));
 const buyPriceCents=selected?.price_editable?digits(buyPrice):selected?.price_per_kg;
 const pending=(selected&&Number.isFinite(kg)&&kg>0&&Number.isSafeInteger(buyPriceCents)&&Number(buyPriceCents)>0)?{articleId:selected.id,name:selected.designation,weightGrams:Math.round(kg*1000),pricePerKg:buyPriceCents,totalCents:Math.round(kg*buyPriceCents)}:null;
 const saleProduct=articles.find(p=>String(p.id)===saleProductId);
 const saleKg=Number(saleWeight.replace(',','.'));
 const salePriceCents=Math.round(Number(salePrice.replace(',','.'))*100);
 const salePending=saleProduct && Number.isFinite(saleKg) && saleKg>0 && Number.isSafeInteger(salePriceCents) && salePriceCents>0 && salePriceCents<=100000000 ? {articleId:saleProduct.id,name:saleProduct.designation,weightGrams:Math.round(saleKg*1000),pricePerKg:salePriceCents,totalCents:Math.round(saleKg*salePriceCents)} : null;
 const total=cart.reduce((sum,c)=>sum+c.totalCents,0)+(pending?.totalCents||0);
 const saleLines=[...saleCart,...(salePending?[salePending]:[])];
 const saleTotal=saleLines.reduce((sum,line)=>sum+line.totalCents,0);
 const addSaleLine=()=>{if(!salePending){fail('Selecione material, quantidade e preço de venda válidos.');return}setSaleCart(v=>[...v,salePending]);setSaleProductId('');setSaleWeight('');setSalePrice('');notify('Material adicionado à venda.');};
 const loadSales=useCallback(async()=>{if(!isAdmin)return;try{setSalesHistory(await api('sales:history',{from:salesFrom,to:salesTo}));}catch(e){fail(e)}},[isAdmin,salesFrom,salesTo]);
 useEffect(()=>{if(tab==='vendas')void loadSales()},[tab,loadSales]);
 useEffect(()=>{if(isAdmin && tab==='config' && configTab==='fiscal'){api('fiscal:company:get').then(setFiscalCompany).catch(fail)}},[tab,configTab,isAdmin]);
 const currentLines=[...cart,...(pending?[pending]:[])];
 const draftTimeline:TimelineEvent[]=[...currentLines.map((l,i)=>({key:`draft-buy-${i}`,title:l.name,when:new Date().toISOString(),status:'pending' as const,kind:'purchase',weightGrams:l.weightGrams,amountCents:l.totalCents})),...(salePending?[{key:'draft-sale',title:salePending.name,when:new Date().toISOString(),status:'pending' as const,kind:'sale',weightGrams:salePending.weightGrams,amountCents:salePending.totalCents}]:[])];

 const addLine=()=>{if(!pending){fail('Confira o produto, o peso e o preço por kg.');return}setCart(v=>[...v,pending]);setWeight('');setProductId('');setBuyPrice('');window.requestAnimationFrame(()=>productFocus.current?.focus());notify('Item registrado. Informe o próximo código do produto.')};
 const openPay=()=>{if(!(tab==='vendas'?saleLines.length:currentLines.length)){fail('Selecione um produto e informe um peso válido.');return}if(!caixa){setTab('caixa');fail('Abra um Caixa Atual antes de validar a pesagem.');return}setMethod('cash');setPaymentOpen(true)};
 useEffect(()=>{if(paymentOpen)paymentCashFocus.current?.focus()},[paymentOpen]);
 const checkout=async()=>{if(busy)return;setBusy(true);try{
   const isSale=tab==='vendas';
   const lines=(isSale?saleLines:currentLines).map(c=>({articleId:c.articleId,weightGrams:c.weightGrams,pricePerKg:c.pricePerKg}));
   const result=await api('checkout',{kind:isSale?'sale':'purchase',lines,clientId:null,buyerCompany:isSale?buyerCompany.trim():null,method});
   setCart([]);setProductId('');setWeight('');setBuyPrice('');setSaleProductId('');setSaleWeight('');setSalePrice('');setSaleCart([]);setBuyerCompany('');setPaymentOpen(false);await refresh();if(isSale)await loadSales();notify(`Operação registrada! Controle ${result.invoice} · ${brl(result.totalCents)}`);
  }catch(e){fail(e)}finally{setBusy(false)}};
 const runDialog=async()=>{if(busy||!dialog)return;setBusy(true);try{
  if(dialog==='period') {setPeriodResult(await api('caixa:period',{from:periodFrom,to:periodTo}));setDialog(null);return}
  if(dialog==='open')await api('caixa:open',{amountCents:digits(amount),number:caixaNumber.trim()?Number(caixaNumber):null});
  else if(dialog==='close')await api('caixa:close');
  else await api('caixa:move',{kind:dialog==='deposit'?'deposit':'withdrawal',amountCents:digits(amount)});
  setDialog(null);setAmount('');setCaixaNumber('');await refresh();await loadPrevious();notify('Caixa atualizado com sucesso');
 }catch(e){fail(e)}finally{setBusy(false)}};
 useEffect(()=>{
  const listener=(ev:KeyboardEvent)=>{
   if(ev.key==='Enter'&&dialog&&!ev.repeat){ev.preventDefault();void runDialog();return}
   if(ev.altKey||ev.ctrlKey||ev.metaKey||ev.shiftKey)return;
   if(ev.key==='F12'&&tab==='weigh'&&!dialog&&!paymentOpen){ev.preventDefault();openPay();return}
   if(ev.key==='F10'&&tab==='vendas'&&!dialog&&!paymentOpen){ev.preventDefault();if(isAdmin)openPay();return}
   const route:Record<string,typeof tab>=isAdmin?{F1:'weigh',F2:'vendas',F3:'caixa',F5:'config',F8:'finance'}:{F1:'weigh',F3:'caixa'};
   if(route[ev.key]){ev.preventDefault();if(dialog||paymentOpen)return;setTab(route[ev.key]);return}
   if(ev.key==='F6'){ev.preventDefault();if(!dialog&&!paymentOpen){setTab('caixa');setCaixaTab('current')}return}
   if(ev.key==='F7'){ev.preventDefault();if(!dialog&&!paymentOpen){setTab('caixa');setCaixaTab('previous')}return}
  };
  window.addEventListener('keydown',listener);return()=>window.removeEventListener('keydown',listener);
 },[tab,dialog,paymentOpen,runDialog,isAdmin,openPay]);
 const save=async(channel:string,data:Obj,clear:()=>void)=>{try{await api(channel,data);await refresh();clear();notify('Registro salvo com sucesso')}catch(e){fail(e)}};
 const remove=async(channel:string,id:number)=>{if(!window.confirm('Confirmar exclusão?'))return;try{await api(channel,id);await refresh();notify('Registro removido')}catch(e){fail(e)}};
 const pdf=async(kind:'sale'|'client',id:number)=>{try{const path=await api('pdf:open',kind,id);notify(`PDF gerado: ${path}`)}catch(e){fail(e)}};
 const readDetail=async(id:number)=>{try{setDetail(await api('sale:detail',id))}catch(e){fail(e)}};
 const readCashDetail=async(id:number)=>{try{setCashDetail(await api('sale:detail',id))}catch(e){fail(e)}};
 const resetFinancial=async()=>{if(resetBusy)return;
   if(!resetConfirmed){addResetLog('Confirme que o banco será salvo antes da limpeza.','error');return}
   if(resetPhrase.trim()!=='ZERAR FINANCEIRO'){addResetLog('Digite exatamente ZERAR FINANCEIRO para confirmar.','error');return}
   setResetBusy(true);setResetLogs([]);
   const log=(message:string,level:'info'|'ok'|'error'='info')=>addResetLog(message,level);
   log('Iniciando manutenção. Os produtos, usuários, categorias e licenças serão preservados.');
   try{
    log('Solicitando backup do SQLite e limpeza transacional ao serviço local…');
    const out=await api('maintenance:reset-financial',{confirmation:resetPhrase.trim()});
    log(`Backup concluído: ${out.backup}`,'ok');
    if(out.removed)log(`Registros removidos: ${Object.entries(out.removed).map(([k,v])=>`${k}: ${v}`).join(' · ')}`,'ok');
    log('Atualizando a tela, o caixa e os relatórios…');
    setCart([]);setSaleCart([]);setSalesHistory([]);setRows([]);setPrevious([]);setDetail(null);setCashDetail(null);setPeriodResult(null);setFinancial(null);
    await refresh();
    await loadFinance();
    log('Limpeza concluída. Confira os indicadores zerados e os cadastros preservados.','ok');
    notify('Dados financeiros zerados com sucesso. Consulte o registro de manutenção.');
    setResetPhrase('');setResetConfirmed(false);
   }catch(e){const description=errorPt(e);log(`A operação não foi concluída: ${description}`,'error');const diagnostic=String((e as Error)?.message||e||'');if(diagnostic!==description)log(`Diagnóstico técnico: ${diagnostic}`,'error');fail(e)}finally{setResetBusy(false)}
 };
 return <div className="shell">
   <div className="workspace">
    <header className="topbar"><div className="headerIdentity"><span className="topbarCaption">Sistema de compra e venda de recicláveis · {identity.username} ({identity.role==='admin'?'Administrador':'Funcionário'})</span></div><button className="secondary" onClick={onLogout}><LogOut size={16}/> Sair</button></header>
    {message&&<div role="status" className="toast success">{message}<button onClick={()=>setMessage('')} aria-label="Fechar aviso"><X size={17}/></button></div>}
    {error&&<div role="alert" className="toast error">{error}<button onClick={()=>setError('')} aria-label="Fechar erro"><X size={17}/></button></div>}
    {tab==='weigh'&&<div className="page weighPage">
      <div className="pageTitle"><h1>Pesagem</h1><span>Compra de materiais recicláveis: selecione o produto, informe o peso e registre o pagamento ao fornecedor.</span></div>
      <div className="weighGrid">
       <section className="panel entryPanel">
        <div className="fieldRow"><div className="field productSelect"><AutoSelect label="Código ou nome do produto" value={productId} onChange={id=>{setProductId(id);const p=articles.find(x=>String(x.id)===id);setBuyPrice(p?(p.price_per_kg/100).toFixed(2).replace('.',','):'')}} inputRef={productFocus} onEnterSelected={()=>weightFocus.current?.focus()} items={articles.filter(p=>p.active).map(p=>({id:String(p.id),code:p.code_article,label:p.designation,meta:`${p.code_article} · ${p.icon||'♻️'} · ${brl(p.price_per_kg)}/kg`}))} placeholder="Digite código (FERRO) ou nome..." preferCode/></div></div>
        {selected&&<div className="productPreview"><span>{icons.includes(selected.icon)?selected.icon:'♻️'}</span><strong>{selected.code_article} · {selected.designation}</strong><small>{brl(selected.price_per_kg)}/kg</small></div>}
        <div className="weighPriceRow"><label className="field">Peso (kg)<div className="weightEntry"><input type="text" inputMode="decimal" placeholder="0,000" aria-label="Peso em quilogramas" ref={weightFocus} value={weight} onChange={e=>setWeight(e.target.value)} onKeyDown={e=>{if(e.key==='Enter'&&!e.repeat){e.preventDefault();if(selected&&Number(weight.replace(',','.'))>0)buyPriceFocus.current?.focus();else fail('Informe um peso válido e selecione o produto.')}}}/><b>kg</b></div></label>
        <label className="field">Preço unitário (R$/kg)<input ref={buyPriceFocus} type="text" inputMode="decimal" aria-label="Preço unitário por quilograma" value={buyPrice} readOnly={!selected?.price_editable} onFocus={e=>{if(selected?.price_editable)e.currentTarget.select()}} onChange={e=>{if(selected?.price_editable)setBuyPrice(cashInput(e.target.value))}} onBlur={()=>{if(selected&&buyPrice.trim())setBuyPrice(cashDisplay(buyPrice))}} onKeyDown={e=>{if(e.key==='Enter'&&!e.repeat){e.preventDefault();addLine()}}} placeholder="0,00"/></label></div>
        <p className="fieldHint">Enter: Código → Peso → Preço/kg → Registrar item → Código. A alteração do preço depende da permissão do produto.</p>
       </section>
       <section className="panel summaryPanel"><h2>Resumo da compra</h2><div className="cartLines">{cart.map((c,i)=><div className="cartLine" key={i}><div><b>{c.name}</b><small>{formatKg(c.weightGrams)} · {brl(c.pricePerKg)}/kg</small></div><strong>{brl(c.totalCents)}</strong><button className="iconButton" onClick={()=>setCart(v=>v.filter((_,j)=>j!==i))} aria-label="Excluir item"><Trash2 size={18}/></button></div>)}{pending&&<div className="cartLine pending"><div><b>{pending.name}</b><small>{formatKg(Math.round(kg*1000))} · {brl(pending.pricePerKg)}/kg · item atual</small></div><strong>{brl(pending.totalCents)}</strong></div>}{!currentLines.length&&<p className="muted">Nenhum item selecionado.</p>}</div><div className="grandTotal"><span>Total</span><strong>{brl(total)}</strong></div><button className="primary full" onClick={openPay} disabled={!currentLines.length}><Check size={21}/>Validar compra e escolher pagamento <kbd>F12</kbd></button></section>
      </div>
    </div>}
    {tab==='vendas'&&<div className="page vendasPage">
      <div className="pageTitle"><h1>Vendas</h1><span>Venda de materiais recicláveis — informe produto, peso e valor por kg.</span></div>
      <div className="weighGrid">
       <section className="panel entryPanel">
        <div className="field productSelect"><AutoSelect label="Produto para venda" value={saleProductId} onChange={id=>{setSaleProductId(id);const product=articles.find(p=>String(p.id)===id);setSalePrice(product?(Number(product.sale_price_per_kg??0)/100).toFixed(2):'')}} items={articles.filter(p=>p.active).map(p=>({id:String(p.id),label:p.designation,meta:p.icon||'♻️'}))} placeholder="Digite o início do produto..."/>
        </div>
        {saleProduct&&<div className="productPreview"><span>{icons.includes(saleProduct.icon)?saleProduct.icon:'♻️'}</span><strong>{saleProduct.designation}</strong></div>}
        <div className="formGrid">
         <label className="field">Peso (kg)<input type="text" inputMode="decimal" placeholder="0,000" value={saleWeight} onChange={e=>setSaleWeight(e.target.value)} aria-label="Peso para venda em kg"/></label>
         <label className="field">Valor por kg (R$)<input type="number" min="0.01" step="0.01" value={salePrice} onChange={e=>setSalePrice(e.target.value)} aria-label="Valor de venda por kg"/></label>
        </div>
        <label className="field">Empresa compradora (opcional)<input type="text" maxLength={150} value={buyerCompany} onChange={e=>setBuyerCompany(e.target.value)} placeholder="Razão social da empresa"/></label>
       </section>
       <section className="panel summaryPanel">
        <h2>Resumo da venda</h2>
        <div className="cartLines">{saleCart.map((line,i)=><div className="cartLine" key={i}><div><b>{line.name}</b><small>{formatKg(line.weightGrams)} · {brl(line.pricePerKg)}/kg</small></div><strong>{brl(line.totalCents)}</strong><button className="iconButton" aria-label="Excluir linha da venda" onClick={()=>setSaleCart(c=>c.filter((_,j)=>j!==i))}><Trash2 size={18}/></button></div>)}{salePending&&<div className="cartLine pending"><div><b>{salePending.name}</b><small>{formatKg(salePending.weightGrams)} · {brl(salePending.pricePerKg)}/kg</small></div><strong>{brl(salePending.totalCents)}</strong></div>}{!saleLines.length&&<p className="muted">Nenhum material selecionado.</p>}</div>
        <div className="grandTotal"><span>Valor total</span><strong>{brl(saleTotal)}</strong></div>
        <button className="secondary full" disabled={!salePending} onClick={addSaleLine}>+ Adicionar material à venda</button>
        <button className="primary full" disabled={!saleLines.length||!caixa} onClick={openPay}><Check size={21}/>Registrar venda e escolher recebimento <kbd>F10</kbd></button>
        {!caixa&&<p className="muted">Abra o Caixa Atual antes de registrar uma venda.</p>}
       </section>
      </div>
      <section className="panel salesHistoryPanel"><h2>Histórico de vendas às empresas</h2><div className="filters"><label>De<input type="date" value={salesFrom} onChange={e=>setSalesFrom(e.target.value)}/></label><label>Até<input type="date" value={salesTo} onChange={e=>setSalesTo(e.target.value)}/></label><button className="secondary" onClick={loadSales}><RefreshCw size={17}/>Atualizar vendas</button></div>
      <div className="tableScroll"><table><thead><tr><th>Venda</th><th>Data</th><th>Empresa</th><th>Materiais</th><th>Quantidade</th><th>Venda (R$)</th><th>Custo (R$)</th><th>Margem bruta</th><th>Desde a anterior</th></tr></thead><tbody>{salesHistory.map(r=><tr className="clickRow" key={r.id} onClick={async()=>{try{setSaleHistoryDetail(await api('sale:detail',r.id))}catch(e){fail(e)}}}><td>{r.invoice_number}</td><td>{date(r.created_at)}</td><td>{r.buyer_company||'—'}</td><td>{r.materials||'—'}</td><td>{formatKg(r.weight_grams)}</td><td>{brl(r.total_cents)}</td><td>{r.missing_cost_lines?'Custo desconhecido':brl(r.cost_cents)}</td><td>{r.missing_cost_lines?'—':brl(r.total_cents-r.cost_cents)}</td><td>{r.previous_at?durationBetween(r.previous_at,r.created_at):'Primeira venda'}</td></tr>)}</tbody></table></div>
      {saleHistoryDetail&&<div className="detail"><h3>Venda {saleHistoryDetail.invoice_number}</h3><p>{date(saleHistoryDetail.created_at)} · {saleHistoryDetail.buyer_company||'Empresa não informada'}</p>{saleHistoryDetail.lines?.map((l:Obj)=><p key={l.id}>{l.designation} — {formatKg(l.weight_grams)} × {brl(l.price_per_kg)}/kg = {brl(l.total_cents)} · custo: {l.cost_per_kg==null?'não informado':brl(Math.round(l.cost_per_kg*l.weight_grams/1000))}</p>)}<strong>Total da venda: {brl(saleHistoryDetail.total_cents)}</strong><button className="secondary" onClick={()=>pdf('sale',saleHistoryDetail.id)}>Abrir PDF</button></div>}</section>
    </div>}
    {tab==='caixa'&&<div className="page"><div className="pageTitle"><h1>Caixa</h1><span>Controle de movimentações e fechamento diário.</span></div>
      <div className="subnav"><button className={caixaTab==='current'?'active':''} onClick={()=>setCaixaTab('current')}>Caixa Atual <kbd>F6</kbd></button><button className={caixaTab==='previous'?'active':''} onClick={()=>setCaixaTab('previous')}>Caixas Anteriores <kbd>F7</kbd></button></div>
      {caixaTab==='current'&&<div className="caixaColumns"><section className="panel caixaPanel"><div className="sectionTitle"><h2>Resumo do Caixa</h2><span className={'status '+(caixa?'open':'closed')}>{caixa?'ABERTO':'FECHADO'}</span></div>
       {caixa?<><div className="statsGrid"><Stat name="Saldo Inicial" value={brl(caixa.initialBalance)}/><Stat name="Total de Vendas" value={brl(caixa.totalSales)}/><Stat name="Total de Compras" value={brl(caixa.totalPurchases||0)}/><Stat name="Valor Retirado" value={brl(caixa.totalWithdrawals)}/><Stat name="Saldo Final (dinheiro)" value={brl(caixa.finalBalance)} emphasis/><Stat name="Lucro/Prejuízo Bruto" value="Não calculado" note="Custos não cadastrados"/><Stat name="Entradas / Suprimentos" value={brl(caixa.totalDeposits)}/></div>
        <p className="muted">Saldo físico = saldo inicial + vendas em dinheiro − compras em dinheiro + depósitos − sangrias. PIX, cartão e transferência não entram no saldo físico.</p>
        <div className="actionRow"><button className="primary" onClick={()=>{setAmount('');setDialog('deposit')}}><ArrowDownLeft size={18}/>Adicionar Dinheiro</button><button className="secondary" onClick={()=>{setAmount('');setDialog('withdrawal')}}><ArrowUpRight size={18}/>Remover Valores / Sangria</button><button className="secondary caixaMainButton" onClick={()=>setDialog('close')}>Fechar Caixa</button></div>
        
        <h3>Movimentações e auditoria</h3><div className="audit">{caixa.historyLogs.slice().reverse().map((x,i)=><div key={i}><b>{x.user}</b> — {x.action}<small>{date(x.timestamp)}</small></div>)}</div>
       </>:<div className="emptyState"><Wallet size={42}/><h3>Nenhum caixa aberto</h3><p>Abra um caixa para receber pagamentos e registrar as pesagens.</p><button className="primary caixaMainButton" onClick={()=>{setAmount('');setDialog('open')}}>Abrir Caixa</button></div>}
      </section><section className="panel caixaTimelinePanel"><h2>Linha do tempo</h2><p className="muted">Pesagens e pagamentos do caixa atual</p><CaixaTimeline events={caixa?.timeline||[]} draft={draftTimeline} onDetail={readCashDetail}/></section></div>}
      {caixaTab==='previous'&&<section className="panel"><div className="sectionTitle"><h2>Histórico de caixas</h2><button className="secondary" onClick={()=>setDialog('period')}>Totalizar Período</button></div><div className="filters"><label>Pesquisar<input value={previousSearch} onChange={e=>setPreviousSearch(e.target.value)} placeholder="Número ou status"/></label><label>De<input type="date" value={periodFrom} onChange={e=>setPeriodFrom(e.target.value)}/></label><label>Até<input type="date" value={periodTo} onChange={e=>setPeriodTo(e.target.value)}/></label><button onClick={loadPrevious}><RefreshCw size={18}/>Atualizar</button></div>
       <div className="tableScroll"><table><thead><tr><th>Ação</th><th>Número</th><th>Abertura</th><th>Fechamento</th><th>Saldo Inicial</th><th>Saldo Final</th></tr></thead><tbody>{previous.map(c=><React.Fragment key={c.id}><tr><td><button className="iconButton" aria-label={`Detalhar caixa ${c.id}`} onClick={()=>setExpanded(expanded===c.id?null:c.id)}><ChevronRight size={18}/></button></td><td>#{c.id}</td><td>{date(c.openedAt)}</td><td>{date(c.closedAt)}</td><td>{brl(c.initialBalance)}</td><td>{brl(c.finalBalance)}</td></tr>{expanded===c.id&&<tr><td colSpan={6}><div className="audit"><strong>Pesagens realizadas e pagas</strong><CaixaTimeline events={c.timeline||[]} onDetail={readCashDetail}/><strong>Rastreabilidade do Caixa #{c.id}</strong>{c.historyLogs.map((l,i)=><div key={i}>Usuário: {l.user} — {l.action} <small>{date(l.timestamp)}</small></div>)}{c.status==='closed'&&!caixa&&<button className="secondary" onClick={async()=>{if(!confirm(`Reabrir caixa ${c.id}?`))return;try{await api('caixa:reopen',c.id);await refresh();await loadPrevious();notify('Caixa reaberto')}catch(e){fail(e)}}}>Reabrir Caixa</button>}</div></td></tr>}</React.Fragment>)}</tbody></table>{previous.length===0&&<p className="muted">Nenhum caixa no período.</p>}</div>
       {periodResult&&<div className="periodTotals"><b>Período {periodResult.from} — {periodResult.to}</b><span>{periodResult.count} caixa(s) · Vendas {brl(periodResult.totalSales)} · Compras {brl(periodResult.totalPurchases||0)} · Dinheiro líquido de compras/vendas {brl(periodResult.cashNet||0)} · Suprimentos {brl(periodResult.totalDeposits)} · Sangrias {brl(periodResult.totalWithdrawals)}</span></div>}
      </section>}
    </div>}
    {cashDetail&&tab==='caixa'&&<div className="overlay"><div className="dialog" role="dialog" aria-modal="true" aria-label="Detalhe da transação"><button className="close" aria-label="Fechar" onClick={()=>setCashDetail(null)}><X/></button><h2>Transação {cashDetail.invoice_number}</h2><p>{date(cashDetail.created_at)} · {cashDetail.kind==='sale'?'Venda':'Compra'}</p><div className="tableScroll"><table><thead><tr><th>Produto</th><th>Peso</th><th>R$/kg</th><th>Total</th></tr></thead><tbody>{cashDetail.lines?.map((l:Obj)=><tr key={l.id}><td>{l.designation}</td><td>{formatKg(l.weight_grams)}</td><td>{brl(l.price_per_kg)}</td><td>{brl(l.total_cents)}</td></tr>)}</tbody></table></div><strong>Total: {brl(cashDetail.total_cents)}</strong><button className="secondary full" onClick={()=>setCashDetail(null)}>Fechar</button></div></div>}
    {tab==='clients'&&<section className="page panel"><h1>Clientes</h1><div className="formGrid"><label>Gênero<select value={clientForm.gender} onChange={e=>setClientForm({...clientForm,gender:e.target.value})}><option value="">Não informado</option><option value="M">Masculino</option><option value="F">Feminino</option><option value="X">Outro</option></select></label><label>Nome<input value={clientForm.first_name} onChange={e=>setClientForm({...clientForm,first_name:e.target.value})}/></label><label>Sobrenome<input value={clientForm.last_name} onChange={e=>setClientForm({...clientForm,last_name:e.target.value})}/></label><label>CPF ou CNPJ<input value={clientForm.document} onChange={e=>setClientForm({...clientForm,document:e.target.value})}/></label></div><button className="primary" onClick={()=>save('clients:save',clientForm,()=>setClientForm({gender:'',first_name:'',last_name:'',document:''}))}>Salvar cliente</button><div className="entityGrid">{clients.map(c=><div className="entity" key={c.id}><b>{c.first_name} {c.last_name}</b><small>{c.document}</small><div className="actionRow"><button onClick={()=>setClientForm(c)}>Editar</button><button onClick={()=>pdf('client',c.id)}><FileText size={17}/>PDF</button><button onClick={()=>remove('clients:delete',c.id)} aria-label="Excluir"><Trash2 size={17}/></button></div></div>)}</div></section>}
    {tab==='config'&&<section className="page panel"><h1>Configuração</h1><div className="subnav"><button className={configTab==='products'?'active':''} onClick={()=>setConfigTab('products')}>Produtos e Categorias</button><button className={configTab==='users'?'active':''} onClick={()=>setConfigTab('users')}>Usuários e Senhas</button><button className={configTab==='control'?'active':''} onClick={()=>setConfigTab('control')}>Controle</button><button className={configTab==='maintenance'?'active':''} onClick={()=>setConfigTab('maintenance')}>Manutenção do banco</button><button className={configTab==='fiscal'?'active':''} onClick={()=>setConfigTab('fiscal')}>Dados Fiscais · NF-e</button></div>
      {isAdmin&&configTab==='fiscal'&&<section className="panel fiscalPanel"><h2>Dados Fiscais · NF-e / NFC-e</h2><p>Etapa A: cadastro fiscal. Nenhuma nota é emitida ou transmitida nesta versão. Confirme NCM, CFOP, CST/CSOSN e regras de seu estado com o contador.</p><div className="formGrid">{([['razao_social','Razão social'],['nome_fantasia','Nome fantasia'],['cnpj','CNPJ (14 caracteres, numérico ou alfanumérico)'],['inscricao_estadual','Inscrição Estadual'],['uf','UF (sigla)'],['codigo_municipio','Código município IBGE (7 dígitos)'],['municipio','Município'],['cep','CEP (8 dígitos)'],['logradouro','Logradouro'],['numero','Número'],['bairro','Bairro'],['complemento','Complemento']] as [string,string][]).map(([key,label])=><label key={key}>{label}<input value={fiscalCompany[key]||''} onChange={e=>setFiscalCompany({...fiscalCompany,[key]:key==='cnpj'?e.target.value.toUpperCase().replace(/[^A-Z0-9]/g,'').slice(0,14):key==='cep'||key==='codigo_municipio'?e.target.value.replace(/\D/g,''):key==='uf'?e.target.value.toUpperCase():e.target.value})}/></label>)}<label>Regime tributário (CRT)<select value={fiscalCompany.crt||'1'} onChange={e=>setFiscalCompany({...fiscalCompany,crt:e.target.value})}><option value="1">1 · Simples Nacional</option><option value="2">2 · Simples Nacional - excesso</option><option value="3">3 · Regime Normal</option><option value="4">4 · MEI (verificar habilitação)</option></select></label><label>Ambiente fiscal<select value={fiscalCompany.ambiente||'homologacao'} onChange={e=>setFiscalCompany({...fiscalCompany,ambiente:e.target.value})}><option value="homologacao">Homologação (testes)</option><option value="producao">Produção (futuro, emissão desabilitada)</option></select></label><label>Certificado e-CNPJ A1 (arquivo externo .pfx/.p12)<div className="actionRow"><input readOnly value={fiscalCompany.certificado_a1_caminho||''} placeholder="Nenhum certificado selecionado"/><button type="button" onClick={async()=>{try{const file=await openDialog({multiple:false,directory:false,filters:[{name:'Certificado A1',extensions:['pfx','p12']}]});if(typeof file==='string')setFiscalCompany({...fiscalCompany,certificado_a1_caminho:file})}catch(e){fail(e)}}}>Selecionar</button><button type="button" onClick={()=>setFiscalCompany({...fiscalCompany,certificado_a1_caminho:''})}>Limpar</button></div></label></div><p className="fiscalNotice">🔒 O arquivo A1 permanece fora do SQLite. Nenhuma senha é armazenada. A existência do arquivo NÃO comprova validade, titularidade ou capacidade de assinatura. O mecanismo de assinatura e a transmissão serão implementados nas etapas B/C.</p><button className="primary" onClick={async()=>{try{await api('fiscal:company:save',fiscalCompany);notify('Dados fiscais salvos. Nenhuma NF-e foi emitida.')}catch(e){fail(e)}}}>Salvar dados fiscais</button></section>}
      {configTab==='products'&&<><h2>Produtos e Categorias</h2><div className="productListSection"><h3>Lista de produtos</h3><div className="entityGrid">{articles.map(p=><div className="entity" key={p.id}><b>{p.icon} {p.designation}</b><small>{p.category_name||'Sem categoria'} · {brl(p.price_per_kg)}/kg · {p.ncm?'NCM '+p.ncm:'⚠ NCM pendente'}</small><div className="actionRow"><button onClick={()=>setProductForm(p)}>Editar</button><button onClick={()=>remove('articles:delete',p.id)}><Trash2 size={17}/></button></div></div>)}</div></div><div className="actionRow"><input value={catName} onChange={e=>setCatName(e.target.value)} placeholder="Nova categoria"/><button onClick={()=>save('categories:save',{name:catName},()=>setCatName(''))}>+ Categoria</button></div><div className="chips">{categories.map(c=><span key={c.id}>{c.name} <button onClick={()=>remove('categories:delete',c.id)}>×</button></span>)}</div><div className="formGrid productFormGrid"><label>Código<input value={productForm.code_article} onChange={e=>setProductForm({...productForm,code_article:e.target.value})}/></label><label>Descrição<input value={productForm.designation} onChange={e=>setProductForm({...productForm,designation:e.target.value})}/></label><label>Preço/kg (R$)<input type="number" step="0.01" min="0" value={(productForm.price_per_kg||0)/100} onChange={e=>setProductForm({...productForm,price_per_kg:Math.round(Number(e.target.value)*100)})}/></label><label>Preço de venda sugerido/kg (R$)<input type="number" min="0" step="0.01" value={productForm.sale_price_per_kg==null?'':productForm.sale_price_per_kg/100} onChange={e=>setProductForm({...productForm,sale_price_per_kg:e.target.value?Math.round(Number(e.target.value)*100):null})}/></label><label>Preço de aquisição estimado/kg (R$)<input type="number" min="0" step="0.01" value={productForm.cost_per_kg==null?'':productForm.cost_per_kg/100} onChange={e=>setProductForm({...productForm,cost_per_kg:e.target.value?Math.round(Number(e.target.value)*100):null})}/></label><label><input type="checkbox" checked={Boolean(productForm.price_editable)} onChange={e=>setProductForm({...productForm,price_editable:e.target.checked})}/> Permitir alterar preço/kg na Pesagem</label><label>NCM (8 dígitos) — obrigatório<input inputMode="numeric" maxLength={8} value={productForm.ncm||''} onChange={e=>setProductForm({...productForm,ncm:e.target.value.replace(/\D/g,'').slice(0,8)})} placeholder="00000000"/></label><label>CEST (7 dígitos, se aplicável)<input maxLength={7} inputMode="numeric" value={productForm.cest||''} onChange={e=>setProductForm({...productForm,cest:e.target.value.replace(/\D/g,'').slice(0,7)})}/></label><label>Origem da mercadoria<select value={productForm.origem_icms||'0'} onChange={e=>setProductForm({...productForm,origem_icms:e.target.value})}>{Array.from({length:9},(_,i)=><option key={i} value={String(i)}>{i}</option>)}</select></label><label>CST ICMS (regime normal)<input maxLength={2} inputMode="numeric" value={productForm.cst_icms||''} onChange={e=>setProductForm({...productForm,cst_icms:e.target.value.replace(/\D/g,'').slice(0,2)})}/></label><label>CSOSN (Simples Nacional)<input maxLength={3} inputMode="numeric" value={productForm.csosn||''} onChange={e=>setProductForm({...productForm,csosn:e.target.value.replace(/\D/g,'').slice(0,3)})}/></label><label>CFOP entrada<input maxLength={4} inputMode="numeric" value={productForm.cfop_entrada||''} onChange={e=>setProductForm({...productForm,cfop_entrada:e.target.value.replace(/\D/g,'').slice(0,4)})}/></label><label>CFOP saída<input maxLength={4} inputMode="numeric" value={productForm.cfop_saida||''} onChange={e=>setProductForm({...productForm,cfop_saida:e.target.value.replace(/\D/g,'').slice(0,4)})}/></label><label>Categoria<select value={productForm.category_id||''} onChange={e=>setProductForm({...productForm,category_id:e.target.value?Number(e.target.value):null})}><option value="">—</option>{categories.map(c=><option key={c.id} value={c.id}>{c.name}</option>)}</select></label><div className="iconField"><span className="iconFieldLabel">Ícone do material</span><MaterialIconPicker value={productForm.icon||'♻️'} onChange={icon=>setProductForm({...productForm,icon})}/></div></div><button className="primary" onClick={()=>save('articles:save',productForm,()=>setProductForm({code_article:'',designation:'',price_per_kg:0,category_id:null,icon:'♻️',price_editable:false,cost_per_kg:null,sale_price_per_kg:null}))}>Salvar produto</button></>}
      {isAdmin&&configTab==='users'&&<section className="panel" style={{marginTop:24}}><h2>Usuários e Senhas</h2><p>Usuários ativos: {users.map(u=>`${u.username} (${u.role})`).join(', ')}</p><div className="formGrid"><label>Nome de usuário<input value={newUser.username} onChange={e=>setNewUser({...newUser,username:e.target.value})}/></label><label>Senha temporária (8 dígitos)<PasswordField numeric value={newUser.password} onChange={password=>setNewUser({...newUser,password})}/></label><label>Perfil<select value={newUser.role} onChange={e=>setNewUser({...newUser,role:e.target.value})}><option value="employee">Funcionário</option><option value="admin">Administrador</option></select></label></div><button className="primary" onClick={async()=>{try{await api('auth:create-user',newUser);setUsers(await api('auth:users'));setNewUser({username:'',password:'',role:'employee'});notify('Usuário criado')}catch(e){fail(e)}}}>Criar usuário</button><h3>Redefinir senha de funcionário</h3><div className="formGrid"><label>Usuário<select value={resetUser} onChange={e=>setResetUser(e.target.value)}><option value="">Selecione</option>{users.filter(u=>u.id!==identity?.id&&u.username!==identity?.username).map(u=><option key={u.id} value={u.id}>{u.username} ({u.role})</option>)}</select></label><label>Nova senha temporária (8 dígitos)<PasswordField numeric value={resetPwd} onChange={setResetPwd}/></label><button onClick={async()=>{try{await api('auth:reset-password',{userId:Number(resetUser),newPassword:resetPwd});setResetPwd('');notify('Senha temporária atualizada')}catch(e){fail(e)}}}>Redefinir senha</button></div><h3>Alterar minha senha</h3><PasswordChange/></section>}
      {configTab==='control'&&<><h2>Histórico de operações</h2><div className="filters"><label>De<input type="date" value={from} onChange={e=>setFrom(e.target.value)}/></label><label>Até<input type="date" value={to} onChange={e=>setTo(e.target.value)}/></label><label>Pesquisar<input value={search} onChange={e=>setSearch(e.target.value)} placeholder="Número, cliente"/></label><button onClick={loadHistory}><Search size={18}/>Buscar</button></div><div className="tableScroll"><table><thead><tr><th>Número</th><th>Data</th><th>Cliente</th><th>Valor</th><th>Recebido</th></tr></thead><tbody>{rows.filter(r=>String(r.invoice_number+' '+r.client_name).toLowerCase().includes(search.toLowerCase())).map(r=><tr key={r.id} onClick={()=>readDetail(r.id)} className="clickRow"><td>{r.invoice_number}</td><td>{date(r.created_at)}</td><td>{r.client_name||'—'}</td><td>{brl(r.total_cents)}</td><td>{brl(r.paid_cents)}</td></tr>)}</tbody></table></div>{detail&&<div className="detail"><h3>Controle {detail.invoice_number}</h3>{detail.lines?.map((l:Obj)=><p key={l.id}>{l.designation} — {formatKg(l.weight_grams)} — {brl(l.total_cents)}</p>)}<b>Total: {brl(detail.total_cents)}</b><button onClick={()=>pdf('sale',detail.id)}><FileText size={18}/>Abrir PDF</button></div>}</>}
      {isAdmin&&configTab==='maintenance'&&<section className="maintenancePanel"><h2>Manutenção do banco · Ambiente de testes</h2>
       <p>Esta função salva uma cópia do SQLite e apaga somente caixas, compras, vendas, pagamentos, histórico financeiro e despesas. Produtos, categorias, usuários, clientes, balanças, configurações e a licença não serão apagados.</p>
       <p><strong>Atenção:</strong> esta operação é destinada exclusivamente ao ambiente de testes. É possível recuperar dados apenas restaurando o backup.</p>
       <label className="maintenanceConfirm"><input type="checkbox" checked={resetConfirmed} disabled={resetBusy} onChange={e=>setResetConfirmed(e.target.checked)}/> Li e entendi que os dados financeiros serão apagados após o backup.</label>
       <label className="field">Digite <strong>ZERAR FINANCEIRO</strong> para confirmar<input value={resetPhrase} disabled={resetBusy} onChange={e=>setResetPhrase(e.target.value)} placeholder="ZERAR FINANCEIRO" autoComplete="off"/></label>
       <button type="button" className="dangerButton" disabled={resetBusy||!resetConfirmed||resetPhrase.trim()!=='ZERAR FINANCEIRO'} onClick={resetFinancial}>{resetBusy?'Executando manutenção…':'Fazer backup e zerar dados financeiros'}</button>
       <div className="maintenanceLog" role="log" aria-live="polite"><h3>Registro detalhado da operação</h3><p className="muted">Acompanhe cada etapa aqui. O processo faz backup antes de apagar os dados.</p>{resetLogs.map((line,i)=><div key={i} className={`maintenanceLogLine ${line.level}`}><time>{line.time}</time><span>{line.message}</span></div>)}{!resetLogs.length&&<p className="muted">Nenhuma operação iniciada nesta sessão.</p>}</div>
       </section>}
    </section>}
   {isAdmin&&tab==='finance'&&<section className="page panel financePage"><h1>Gestão Financeira</h1><p>Receitas, custos de aquisição, despesas operacionais e resultado estimado — R$</p><div className="filters"><label>De<input type="date" value={from} onChange={e=>setFrom(e.target.value)}/></label><label>Até<input type="date" value={to} onChange={e=>setTo(e.target.value)}/></label><label>Agrupar<select value={reportPeriod} onChange={e=>setReportPeriod(e.target.value)}><option value="day">Dia</option><option value="week">Semana</option><option value="month">Mês</option></select></label><button onClick={loadFinance}>Atualizar</button></div>{financial&&<><div className="statsGrid"><Stat name="Faturamento" value={brl(financial.revenueCents)}/><Stat name="CMV (custo conhecido)" value={brl(financial.costCents)}/><Stat name="Lucro bruto estimado" value={financial.complete?brl(financial.grossCents):'—'} note={!financial.complete?'Custos incompletos':undefined}/><Stat name="Despesas" value={brl(financial.expensesCents)}/><Stat name="Resultado líquido estimado" value={financial.complete?brl(financial.netCents):'—'}/><Stat name="Margem bruta" value={financial.complete&&financial.revenueCents?`${(100*financial.grossCents/financial.revenueCents).toFixed(1)}%`:'—'}/></div>{!financial.complete&&<div role="alert" className="toast error">{financial.missingCostLines} linha(s) sem custo de aquisição. Os resultados e margens não estão completos.</div>}<h2>Rentabilidade por produto</h2><div className="tableScroll"><table><thead><tr><th>Produto</th><th>Vendas</th><th>CMV</th><th>Lucro bruto</th><th>Margem</th><th>Status</th></tr></thead><tbody>{financial.products?.map((p:Obj)=><tr key={p.id}><td>{p.designation}</td><td>{brl(p.revenue_cents)}</td><td>{brl(p.known_cost_cents)}</td><td>{p.unknown_lines?'—':brl(p.revenue_cents-p.known_cost_cents)}</td><td>{p.unknown_lines||!p.revenue_cents?'—':`${(100*(p.revenue_cents-p.known_cost_cents)/p.revenue_cents).toFixed(1)}%`}</td><td>{p.unknown_lines?'Custo incompleto':'Calculado'}</td></tr>)}</tbody></table></div><h2>Relatório por {reportPeriod==='day'?'dia':reportPeriod==='week'?'semana':'mês'}</h2><div className="tableScroll"><table><thead><tr><th>Período</th><th>Vendas</th><th>CMV</th><th>Bruto</th><th>Despesas</th><th>Líquido</th></tr></thead><tbody>{financial.byPeriod?.map((r:Obj)=><tr key={r.period}><td>{r.period}</td><td>{brl(r.revenueCents)}</td><td>{brl(r.costCents)}</td><td>{r.missingCostLines?'—':brl(r.grossCents)}</td><td>{brl(r.expensesCents)}</td><td>{r.missingCostLines?'—':brl(r.netCents)}</td></tr>)}</tbody></table></div></>}<h2>Categorias de despesas</h2><div className="actionRow"><input placeholder="Nova categoria" value={expenseCategoryName} onChange={e=>setExpenseCategoryName(e.target.value)}/><button onClick={async()=>{try{await api('finance:category-save',{name:expenseCategoryName});setExpenseCategoryName('');await loadFinance()}catch(e){fail(e)}}}>Adicionar categoria</button></div><h2>Registrar despesa</h2><div className="formGrid"><label>Categoria<select value={expense.categoryId} onChange={e=>setExpense({...expense,categoryId:e.target.value})}><option value="">Selecione</option>{expenseCategories.map(c=><option value={c.id} key={c.id}>{c.name}</option>)}</select></label><label>Descrição<input value={expense.description} onChange={e=>setExpense({...expense,description:e.target.value})}/></label><label>Valor (R$)<input value={expense.value} onChange={e=>setExpense({...expense,value:e.target.value})}/></label><label>Data<input type="date" value={expense.date} onChange={e=>setExpense({...expense,date:e.target.value})}/></label></div><button className="primary" onClick={async()=>{try{await api('finance:expense-add',{categoryId:Number(expense.categoryId),description:expense.description,amountCents:digits(expense.value),date:expense.date});setExpense({...expense,value:'',description:''});await loadFinance();notify('Despesa registrada')}catch(e){fail(e)}}}>Salvar despesa</button></section>}
   </div>
   <aside className="leftNav"><div className="sideBrand"><div className="brandMark" aria-hidden>♻</div><div><strong>Recycling System</strong><span>INDUSTRIAL</span></div></div><div className="navLabel">MENU PRINCIPAL</div>{[{id:'weigh',label:'Pesagem',key:'F1',icon:ShoppingBasket},{id:'vendas',label:'Controle',key:'F2',icon:ArrowUpRight},{id:'caixa',label:'Caixa',key:'F3',icon:Wallet},{id:'finance',label:'Gestão Financeira',key:'F8',icon:BarChart3},{id:'config',label:'Configuração',key:'F5',icon:Settings}].map(n=><button key={n.id} disabled={!isAdmin&&!['weigh','caixa'].includes(n.id)} title={!isAdmin&&!['weigh','caixa'].includes(n.id)?'Acesso exclusivo ao administrador':n.label} className={tab===n.id?'selected':''} onClick={()=>{if(isAdmin||n.id==='weigh'||n.id==='caixa')setTab(n.id as typeof tab)}}><n.icon size={23}/><span>{n.label}</span><kbd>{n.key}</kbd></button>)}<div className="navShortcuts"><button onClick={()=>{setTab('caixa');setCaixaTab('current')}}>Caixa Atual <kbd>F6</kbd></button><button onClick={()=>{setTab('caixa');setCaixaTab('previous')}}>Anteriores <kbd>F7</kbd></button></div><div className="navFooter"><span className={'sideCaixa '+(caixa?'isOpen':'')}>{caixa?`Caixa Nº ${caixa.id} · Aberto`:'Caixa fechado'}</span><span className="sideVersion">Versão 8.1.0 · V8-NotaFiscal</span></div></aside>
   {paymentOpen&&<div className="overlay" role="presentation"><div className="dialog" role="dialog" aria-modal="true" aria-label="Escolher pagamento" onKeyDown={e=>{if(e.key==='Enter'&&!e.repeat&&!e.shiftKey&&!e.ctrlKey&&!e.altKey&&!e.metaKey){e.preventDefault();e.stopPropagation();if(!busy)void checkout()}}}><button className="close" aria-label="Fechar" onClick={()=>setPaymentOpen(false)}><X/></button><h2>Forma de pagamento</h2><p>{tab==='weigh'?'Escolha como pagar ao fornecedor.':'Selecione como o cliente realizará o pagamento.'}</p><div className="paymentTotal">{brl(tab==='vendas'?saleTotal:total)}</div><p className="paymentHint">Use Tab ou clique para escolher o pagamento. Pressione Enter para confirmar e fechar esta janela.</p><div className="paymentMethods">{methods.map(m=><button key={m.code} type="button" ref={m.code==='cash'?paymentCashFocus:undefined} aria-pressed={method===m.code} className={method===m.code?'chosen':''} onClick={()=>setMethod(m.code)}><strong>{m.icon}</strong><span>{m.label}</span></button>)}</div><button className="primary full" disabled={busy} onClick={checkout}>Confirmar {tab==='weigh'?'compra':'venda'} · {brl(tab==='vendas'?saleTotal:total)}</button></div></div>}
   {dialog&&<div className="overlay" role="presentation"><div className="dialog" role="dialog" aria-modal="true" aria-label="Operação de caixa"><button className="close" aria-label="Fechar" onClick={()=>setDialog(null)}><X/></button><h2>{({open:'Abrir Caixa',deposit:'Adicionar Dinheiro',withdrawal:'Remover Valores / Sangria',close:'Fechar Caixa',period:'Totalizar Período'})[dialog]}</h2>{dialog==='close'?<p>Confirma o fechamento do Caixa Atual?</p>:dialog==='period'?<div className="formGrid"><label>Data inicial<input type="date" value={periodFrom} onChange={e=>setPeriodFrom(e.target.value)}/></label><label>Data final<input type="date" value={periodTo} onChange={e=>setPeriodTo(e.target.value)}/></label></div>:<div className="cashOpenFields">{dialog==='open'&&<label>Número do caixa (opcional)<input type="text" inputMode="numeric" value={caixaNumber} onChange={e=>setCaixaNumber(e.target.value.replace(/\D/g,''))} placeholder="Automático" aria-label="Número do caixa"/></label>}<label>Valor (R$)<input autoFocus inputMode="decimal" value={amount} onChange={e=>setAmount(cashInput(e.target.value))} onFocus={e=>e.currentTarget.select()} onBlur={()=>setAmount(cashDisplay(amount))} placeholder="0,00"/></label></div>}<button className="primary full" disabled={busy} onClick={runDialog}>Salvar</button></div></div>}
  </div>;
}
type Choice={id:string;label:string;meta?:string;code?:string};
function AutoSelect({value,onChange,items,placeholder,label,allowEmpty=false,inputRef,onEnterSelected,preferCode=false}:{value:string;onChange:(v:string)=>void;items:Choice[];placeholder:string;label:string;allowEmpty?:boolean;inputRef?:React.RefObject<HTMLInputElement|null>;onEnterSelected?:()=>void;preferCode?:boolean}){
 const [query,setQuery]=useState(''),[open,setOpen]=useState(false),[highlight,setHighlight]=useState(0);
 const id=React.useId(); const ref=useRef<HTMLDivElement>(null);
 const normalize=(s:string)=>s.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLocaleLowerCase('pt-BR').trim();
 const selected=items.find(i=>i.id===value);
 useEffect(()=>{if(!open)setQuery((preferCode?selected?.code:selected?.label)||'')},[selected?.label,open]);
 const matched=items.filter(i=>normalize(i.label).startsWith(normalize(query))||(preferCode&&normalize(i.code||'').startsWith(normalize(query)))).sort((a,b)=>(normalize(a.code||'')===normalize(query)?-1:0)-(normalize(b.code||'')===normalize(query)?-1:0)).slice(0,8);
 const choose=(choice?:Choice)=>{onChange(choice?.id||'');setQuery((preferCode?choice?.code:choice?.label)||'');setOpen(false);setHighlight(0)};
 return <div className="combo" ref={ref}>
  <label htmlFor={id}>{label}{allowEmpty?' (opcional)':''}</label>
  <input ref={inputRef} id={id} role="combobox" aria-expanded={open} aria-controls={`${id}-list`} aria-autocomplete="list" placeholder={placeholder} autoComplete="off" value={query}
   onFocus={()=>{setOpen(false);setHighlight(0)}} onBlur={()=>setTimeout(()=>setOpen(false),120)}
   onChange={e=>{setQuery(e.target.value);onChange('');setOpen(Boolean(e.target.value.trim()));setHighlight(0)}}
   onKeyDown={e=>{if(!open&&e.key==='ArrowDown'){e.preventDefault();setOpen(true)}else if(open&&e.key==='ArrowDown'){e.preventDefault();setHighlight(v=>Math.min(v+1,matched.length-1))}else if(open&&e.key==='ArrowUp'){e.preventDefault();setHighlight(v=>Math.max(v-1,0))}else if(e.key==='Enter'){e.preventDefault();if(open&&matched[highlight]&&query.trim()){choose(matched[highlight]);onEnterSelected?.()}else if(value){setOpen(false);onEnterSelected?.()}}else if(e.key==='Escape'){setOpen(false)}}}/>
  {open&&<div className="comboList" id={`${id}-list`} role="listbox">
   {allowEmpty&&<button type="button" role="option" aria-selected={!value} onMouseDown={e=>e.preventDefault()} onClick={()=>choose()}>Sem cliente</button>}
   {matched.map((item,i)=><button type="button" role="option" aria-selected={value===item.id} className={highlight===i?'highlight':''} key={item.id} onMouseDown={e=>e.preventDefault()} onClick={()=>choose(item)}><strong>{item.label}</strong><small>{item.meta}</small></button>)}
   {matched.length===0&&<p>Nenhum produto encontrado. Confira o código ou nome.</p>}
  </div>}
 </div>
}
type TimelineEvent={key?:string;id?:number;title:string;when:string;status:'paid'|'pending';kind:string;weightGrams:number;amountCents:number;method?:string;invoice?:string};
function CaixaTimeline({events,draft=[],onDetail}:{events:TimelineEvent[];draft?:TimelineEvent[];onDetail?:(id:number)=>void}){
 const all=[...draft,...events].sort((a,b)=>b.when.localeCompare(a.when));
 return <div className="timeline" aria-label="Linha do tempo das pesagens">
  {all.length===0&&<p className="muted">Nenhuma pesagem registrada neste caixa.</p>}
  {all.map((e,i)=><div className={'timelineEvent '+(e.status==='pending'?'pending':'paid')} key={e.key||`${e.id}-${i}`}>
   <div className="timelineStamp">{new Date(e.when).toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'})} · {date(e.when)} <span className="timelineStatus">{e.status==='pending'?'Em andamento':'Pago'}</span></div>
   <strong>{e.title}</strong><div className="timelineInfo">{e.kind==='sale'?'Venda':'Compra'} · {formatKg(e.weightGrams)} · {brl(e.amountCents)}</div>
   {e.status==='paid'&&<><small>{e.invoice||''} {e.method?`· ${e.method}`:''}</small>{e.id&&onDetail&&<button type="button" className="secondary" onClick={()=>onDetail(e.id!)}>Ver produtos, pesos e valores</button>}</>}
  </div>)}
 </div>
}
function Stat({name,value,note,emphasis}: {name:string;value:string;note?:string;emphasis?:boolean}){return <div className={'stat '+(emphasis?'emphasis':'')}><span>{name}</span><strong>{value}</strong>{note&&<small>{note}</small>}</div>}
type PasswordFieldProps={value:string;onChange:(value:string)=>void;onKeyDown?:React.KeyboardEventHandler<HTMLInputElement>;numeric?:boolean;placeholder?:string};
function PasswordField({value,onChange,onKeyDown,numeric=false,placeholder}:PasswordFieldProps){
 const [visible,setVisible]=useState(false);
 return <span className="passwordField">
  <input type={visible?'text':'password'} value={value} onChange={e=>onChange(numeric?e.target.value.replace(/\D/g,'').slice(0,8):e.target.value)} onKeyDown={onKeyDown} inputMode={numeric?'numeric':undefined} pattern={numeric?'[0-9]{8}':undefined} maxLength={numeric?8:undefined} autoComplete="off" placeholder={placeholder}/>
  <button type="button" className="passwordEye" aria-label={visible?'Ocultar senha':'Mostrar senha'} title={visible?'Ocultar senha':'Mostrar senha'} aria-pressed={visible} onClick={()=>setVisible(v=>!v)}>{visible?<EyeOff size={21}/>:<Eye size={21}/>}</button>
 </span>;
}
function PasswordChange({forced=false,onSuccess}:{forced?:boolean;onSuccess?:()=>void}){
 const [oldPassword,setOld]=useState(''),[newPassword,setNew]=useState(''),[error,setError]=useState('');
 return <div className="formGrid"><label>Senha atual<PasswordField value={oldPassword} onChange={setOld}/></label><label>Nova senha (8 dígitos)<PasswordField numeric value={newPassword} onChange={setNew}/></label><button className="primary" onClick={async()=>{try{await api('auth:change-password',{oldPassword,newPassword});setOld('');setNew('');setError('');onSuccess?.()}catch(e){setError(errorPt(e))}}}>{forced?'Definir nova senha':'Trocar senha'}</button>{error&&<p role="alert">{error}</p>}</div>;
}
function LoginGate(){
 const [identity,setIdentity]=useState<Obj|null>(null),[user,setUser]=useState('admin'),[password,setPassword]=useState(''),[err,setErr]=useState('');
 const [loggingIn,setLoggingIn]=useState(false);
 const login=async()=>{
  if(loggingIn)return;
  if(!user.trim()||!password){setErr('Informe o usuário e a senha para entrar.');return;}
  setErr('');setLoggingIn(true);
  try{const result=await api('auth:login',{username:user.trim(),password});setSessionToken(result.session);setIdentity(result);setPassword('')}
  catch(e){setErr(errorPt(e))}
  finally{setLoggingIn(false)}
 };
 const logout=()=>{void api('auth:logout').finally(()=>{setSessionToken(null);setIdentity(null)})};
 if(!identity)return <main className="activationScreen loginScreen"><div className="activationCard loginCard">
  <div className="loginBrand"><div className="loginBrandMark" aria-hidden="true">♻</div><div className="loginBrandName"><strong>Recycling<br/>System</strong><span>INDUSTRIAL</span></div><span className="loginVersion">V8-NotaFiscal</span></div>
  <div className="loginIntro"><h1>Entrar no sistema</h1><p>Acesse o Recycling System Industrial</p></div>
  <label>Usuário<span className="loginInputWithIcon"><UserRound size={20}/><input autoFocus autoComplete="username" disabled={loggingIn} value={user} placeholder="Digite seu usuário" onChange={e=>setUser(e.target.value)} onKeyDown={e=>{if(e.key==='Enter')void login()}}/></span></label>
  <label>Senha<span className="loginInputWithIcon"><LockKeyhole size={20}/><PasswordField numeric value={password} onChange={setPassword} onKeyDown={e=>{if(e.key==='Enter')void login()}} placeholder="Digite sua senha"/></span></label>
  {err&&<p className="loginError" role="alert">{err}</p>}
  <button className="primary full loginSubmit" disabled={loggingIn} aria-busy={loggingIn} onClick={()=>void login()}>{loggingIn?<><LoaderCircle className="loginSpinner" size={25}/><span>Validando credenciais...</span></>:<>Entrar</>}</button>
  <div className="loginFootnote">Acesso protegido por licença · Versão 8.1.0 · V8-NotaFiscal</div>
 </div></main>;
 if(identity.mustChange)return <main className="activationScreen"><div className="activationCard"><h1>Alteração obrigatória de senha</h1><p>Defina uma senha própria antes de usar o sistema.</p><PasswordChange forced onSuccess={()=>setIdentity({...identity,mustChange:false})}/></div></main>;
 return <App identity={identity} onLogout={logout}/>;
}
function LicenseGate(){
 const [status,setStatus]=useState<Obj|null>(null);
 const [error,setError]=useState('');
 const [working,setWorking]=useState(false);
 const [mode,setMode]=useState<'copy'|'external'>('copy');
 const refresh=()=>api('license:status').then(setStatus).catch((e)=>setError(String(e)));
 useEffect(()=>{void refresh()},[]);
 const onFile=async(e:React.ChangeEvent<HTMLInputElement>)=>{
  const file=e.target.files?.[0]; if(!file)return;
  setWorking(true);setError('');
  try { const text=await file.text();await api('license:install',text);await refresh(); }
  catch(err){setError(String(err))}finally{setWorking(false);e.target.value=''}
 };
 const selectExternal=async()=>{
  setError('');setWorking(true);
  try {
   const chosen=await openDialog({multiple:false,directory:false,filters:[{name:'Licença Recycling System',extensions:['lic']}]});
   if(typeof chosen==='string') {await api('license:use-external',chosen);await refresh();}
  }catch(err){setError(String(err))}finally{setWorking(false)}
 };
 if(!status)return <main className="activationScreen"><div className="licenseLoading" role="status"><LoaderCircle className="loginSpinner" size={36}/><span>Verificando a licença...</span></div></main>;
 if(status.valid)return <LoginGate/>;
 return <main className="activationScreen"><div className="activationCard">
  <div className="activationLogo">♻</div>
  <h1>Recycling System Industrial</h1><p>Ativação da licença · Versão 8.1.0 · V8-NotaFiscal</p>
  <p>Este computador precisa de uma licença válida. Solicite ao fornecedor um arquivo .lic usando o código abaixo.</p>
  <label>Código deste computador</label>
  <div className="activationMachine"><code>{status.machineId}</code><button onClick={()=>navigator.clipboard.writeText(status.machineId)}>Copiar</button></div>
  <p>Estado: <strong>{status.code}</strong></p>
  <fieldset className="licenseModes"><legend>Onde está o arquivo de licença?</legend>
   <label><input type="radio" name="licenseMode" checked={mode==='copy'} onChange={()=>setMode('copy')}/> Importar e guardar no aplicativo (recomendado)</label>
   <label><input type="radio" name="licenseMode" checked={mode==='external'} onChange={()=>setMode('external')}/> Usar um arquivo no local escolhido</label>
  </fieldset>
  {mode==='copy' ? <label className="activationFile">Procurar arquivo .lic para importar<input type="file" accept=".lic" disabled={working} onChange={onFile}/></label>
   : <button className="primary" disabled={working} onClick={selectExternal}>Procurar arquivo .lic no computador…</button>}
  {mode==='external'&&<small>O arquivo será verificado neste caminho a cada inicialização. Se você mover ou remover o arquivo ou o dispositivo USB, a ativação deixará de funcionar.</small>}
  <div className="licenseLocation"><strong>Local configurado:</strong><code>{status.licenseLocation||'Nenhum'}</code></div>
  {error&&<p role="alert" className="activationError">{error}</p>}
  <button onClick={refresh} disabled={working}>Verificar novamente</button>
  <small>Se a licença expirar, seus dados locais permanecerão preservados.</small>
 </div></main>
}
createRoot(document.getElementById('root')!).render(<LicenseGate/>);
