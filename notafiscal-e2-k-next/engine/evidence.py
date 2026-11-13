"""Offline evidence status ledger. Cannot approve a SEFAZ result automatically."""
from pathlib import Path
import json,hashlib
STAGES={'E2':['xmldsig_profile','a1_identity'], 'F':['official_schema','tax_engine'], 'G':['sefaz_homolog_response','endpoint_verified'], 'H':['danfe_pdf','events'], 'I':['pesagem','controle','caixa'], 'J':['accountant_approval','credential','security_audit'], 'K':['linux_installer','windows_installer','restore_test']}

def make_report():
    return {'production_enabled':False,'sefaz_authorized':False,'gates':{stage:{k:{'state':'PENDENTE','evidence_sha256':None} for k in keys} for stage,keys in STAGES.items()}}

def record_submission(report,stage,key,filepath):
    if stage not in STAGES or key not in STAGES[stage]:raise ValueError('UNKNOWN_GATE')
    p=Path(filepath)
    if not p.is_file(): raise ValueError('EVIDENCE_NOT_FOUND')
    report['gates'][stage][key]={'state':'SUBMETIDA_PARA_ANALISE','evidence_sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
    report['production_enabled']=False
    report['sefaz_authorized']=False
    return report

def save(report,path):Path(path).write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
