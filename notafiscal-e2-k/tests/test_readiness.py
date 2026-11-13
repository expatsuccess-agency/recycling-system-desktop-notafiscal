from tools.notafiscal.readiness import evaluate

def test_no_evidence_all_blocked():
    r=evaluate({})
    assert not r['todas_as_etapas_concluidas']
    assert not r['emissao_producao_ativada_por_este_programa']
    assert all(not item['completo'] for item in r['etapas'].values())

def test_one_evidence_does_not_open_production():
    r=evaluate({'assinatura_local_verificada':True})
    assert not r['etapas']['E2']['completo']
    assert not r['emissao_producao_ativada_por_este_programa']
