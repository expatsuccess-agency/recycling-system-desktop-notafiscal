#!/usr/bin/env python3
"""Checklist objetiva E2-K, conservadora: nunca libera produção automaticamente."""
import argparse, json
from pathlib import Path

STAGES = {
 'E2': ['assinatura_local_verificada','certificado_icp_brasil_validado','perfil_xmldsig_nfe_validado'],
 'F': ['xml_nfe_modelo55_gerado','xsd_oficial_validado','tributacao_aprovada_contador'],
 'G': ['servicos_svrs_wsdl_verificados','homologacao_autorizacao_com_protocolo','recuperacao_idempotente_validada'],
 'H': ['danfe_conforme_moc_validado','eventos_oficiais_homologados','armazenamento_xml_protocolo_validado'],
 'I': ['pesagem_integrada','controle_integrado','financeiro_conciliado_sem_perda'],
 'J': ['contabilidade_aprovou','credenciamento_confirmado','seguranca_e_backup_testados'],
 'K': ['linux_release_testado','windows_release_testado','rollback_e_migracao_testados']
}

def evaluate(evidence):
    results = {}
    for stage, keys in STAGES.items():
        missing = [key for key in keys if evidence.get(key) is not True]
        results[stage] = {'completo': not missing, 'pendencias': missing}
    return {'etapas':results,'todas_as_etapas_concluidas':all(x['completo'] for x in results.values()),
            'emissao_producao_ativada_por_este_programa':False}

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--evidencias',help='JSON com evidências verificadas externamente. Ausência = pendente.')
    p.add_argument('--out',help='Salvar relatório local JSON')
    a=p.parse_args()
    evidence=json.loads(Path(a.evidencias).read_text()) if a.evidencias else {}
    if not isinstance(evidence,dict):raise ValueError('JSON de evidências deve ser um objeto')
    result=evaluate(evidence)
    content=json.dumps(result,indent=2,ensure_ascii=False)
    if a.out:Path(a.out).write_text(content+'\n')
    print(content)
if __name__=='__main__':main()
