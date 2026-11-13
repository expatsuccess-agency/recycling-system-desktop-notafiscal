#!/usr/bin/env python3
"""Gates de evidência E2-J; jamais ativa produção nem afirma homologação automaticamente."""
from pathlib import Path
import json, argparse,hashlib,datetime,sys
BASE=Path(__file__).resolve().parent
REQUIREMENTS={
'E2': [('assinatura_nfe_validada_por_verificador_independente','Prova de assinatura XMLDSig NF-e por verificador independente'),('certificado_icp_brasil_cnpj_revogacao','Evidência de certificado, vínculo CNPJ e cadeia/revogação')],
'F': [('xml_xsd_esquemas_oficiais','Relatório XSD usando pacote oficial vigente'),('regras_fiscais_aprovadas','Regras tributárias validadas pelo contador')],
'G': [('homologacao_sefaz_resposta_oficial','Protocolo de homologação SEFAZ/SVRS (dados sensíveis removidos)'),('credenciamento_confirmado','Credenciamento verificado junto ao órgão competente'),('retries_consulta_idempotencia','Cenários de timeout/reconsulta/rejeição')],
'H': [('danfe_protocolos_eventos','DANFE e protocolo autorizações/eventos homologação'),('cancelamento_inutilizacao','Testes de cancelamento/inutilização quando aplicáveis')],
'I': [('pesagem_controle_financeiro','Recette integrada Pesagem, Controle e Financeiro'),('sqlite_backup_restore','Backup e restauração de base de testes')],
'J': [('parecer_contador','Parecer formal do contador para o estabelecimento/regime'),('auditoria_seguranca','Auditoria segurança e proteção de dados'),('aprovacao_operacional','Aprovação homologação e go-live assinada')],
}
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--report',default=str(BASE/'RELATORIO_E2_J.json'));args=ap.parse_args()
    results=[]
    for stage,pairs in REQUIREMENTS.items():
        for name,desc in pairs:
            evidence=BASE/'evidencias'/stage/(name+'.pdf')
            # Evidence not validated merely because present: report as 'submetida' only, never approved.
            state='SUBMETIDA_PARA_ANALISE' if evidence.is_file() and evidence.stat().st_size>0 else 'PENDENTE'
            results.append({'etapa':stage,'item':name,'descricao':desc,'situacao':state, 'arquivo':str(evidence.relative_to(BASE)) if state!='PENDENTE' else None,
            'sha256':hashlib.sha256(evidence.read_bytes()).hexdigest() if state!='PENDENTE' else None})
    doc={'gerado_em_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'situacao':'BLOQUEADO_PARA_EMISSAO',
         'nota':'Arquivos submetidos não são validados automaticamente. Aprovação fiscal/SEFAZ exige conferência humana e técnica independente.',
         'itens':results,'itens_pendentes':sum(x['situacao']=='PENDENTE' for x in results),
         'emissao_producao_habilitada':False,'homologacao_sefaz_confirmada_automaticamente':False}
    Path(args.report).write_text(json.dumps(doc,indent=2,ensure_ascii=False)+'\n')
    for st in REQUIREMENTS:
        items=[x for x in results if x['etapa']==st]
        print(f'{st}: '+', '.join(x['situacao'] for x in items))
    print('EMISSAO_PRODUCAO=BLOQUEADA')
    print('RELATORIO='+args.report)
if __name__=='__main__':main()
