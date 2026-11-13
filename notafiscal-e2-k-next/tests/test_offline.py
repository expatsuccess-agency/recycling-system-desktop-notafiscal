import unittest,sys,tempfile,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'engine'))
from nfe_draft import build_draft,DraftError,validate_external_xsd
from evidence import make_report,record_submission
from lxml import etree

def valid():
 return {'production':False,'tax_rules_approved':True,'ide':{'cUF':'33','cNF':'00000001','natOp':'VENDA','mod':'55','serie':'1','nNF':'1','dhEmi':'2026-11-13T12:00:00-03:00','tpNF':'1','idDest':'1','cMunFG':'3304557','tpImp':'1','tpEmis':'1','cDV':'0','tpAmb':'2','finNFe':'1','indFinal':'0','indPres':'9','procEmi':'0','verProc':'DEV'},'emit':{'CNPJ':'12345678000190','xNome':'Empresa TESTE','CRT':'1'},'dest':{'CNPJ':'12345678000190','xNome':'Cliente TESTE'},'items':[{'cProd':'ALU','xProd':'Aluminio','NCM':'76020000','CFOP':'5102','uCom':'KG','qCom':'10.000','vUnCom':'2.00','vProd':'20.00'}]}
class Tests(unittest.TestCase):
 def test_draft_is_xml_namespace(self):
  root=etree.fromstring(build_draft(valid()));self.assertTrue(root.tag.endswith('NFe'))
 def test_production_blocked(self):
  x=valid();x['production']=True
  with self.assertRaisesRegex(DraftError,'PRODUCAO_DESATIVADA'):build_draft(x)
 def test_rj_only(self):
  x=valid();x['ide']['cUF']='35'
  with self.assertRaisesRegex(DraftError,'APENAS_UF_RJ'):build_draft(x)
 def test_homolog_only(self):
  x=valid();x['ide']['tpAmb']='1'
  with self.assertRaisesRegex(DraftError,'APENAS_HOMOLOGACAO'):build_draft(x)
 def test_tax_approval_required(self):
  x=valid();x['tax_rules_approved']=False
  with self.assertRaisesRegex(DraftError,'REGRAS_FISCAIS_NAO_APROVADAS'):build_draft(x)
 def test_regime_required(self):
  x=valid();x['emit'].pop('CRT')
  with self.assertRaisesRegex(DraftError,'REGIME_TRIBUTARIO_INDEFINIDO'):build_draft(x)
 def test_items_required(self):
  x=valid();x['items']=[]
  with self.assertRaisesRegex(DraftError,'ITENS_OBRIGATORIOS'):build_draft(x)
 def test_ncm(self):
  x=valid();x['items'][0]['NCM']='BAD'
  with self.assertRaisesRegex(DraftError,'FORMATO_INVALIDO:NCM'):build_draft(x)
 def test_negative_rejected(self):
  x=valid();x['items'][0]['qCom']='-10'
  with self.assertRaisesRegex(DraftError,'VALOR_INVALIDO:qCom'):build_draft(x)
 def test_cannot_approve_evidence(self):
  r=make_report();self.assertFalse(r['production_enabled'])
  with tempfile.TemporaryDirectory() as d:
   p=Path(d)/'evidence.txt';p.write_text('TESTE')
   record_submission(r,'G','sefaz_homolog_response',p)
   self.assertEqual(r['gates']['G']['sefaz_homolog_response']['state'],'SUBMETIDA_PARA_ANALISE')
   self.assertFalse(r['sefaz_authorized'])
 def test_external_xsd_required(self):
  with self.assertRaises(Exception):validate_external_xsd(build_draft(valid()),'/file/not/existing.xsd')
if __name__=='__main__':unittest.main()
