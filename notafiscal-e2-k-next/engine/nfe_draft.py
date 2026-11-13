"""Strict offline NF-e 55 *draft* builder.
Never an authorized fiscal document. No arbitrary tax defaults, no network.
Requires externally approved tax values and official XSD validation before use.
"""
from __future__ import annotations
import re
from decimal import Decimal, InvalidOperation
from lxml import etree

NFE='http://www.portalfiscal.inf.br/nfe'
NS=lambda t:f'{{{NFE}}}{t}'

class DraftError(ValueError): pass

def required(obj, key):
    value=obj.get(key)
    if value is None or str(value).strip()=='': raise DraftError(f'CAMPO_OBRIGATORIO:{key}')
    return str(value).strip()

def numeric(obj,key,length):
    v=required(obj,key)
    if not re.fullmatch(r'\d{' + str(length) + r'}',v): raise DraftError(f'FORMATO_INVALIDO:{key}')
    return v

def decimal(obj,key):
    s=required(obj,key)
    try: v=Decimal(s)
    except InvalidOperation: raise DraftError(f'VALOR_INVALIDO:{key}')
    if not v.is_finite() or v<0: raise DraftError(f'VALOR_INVALIDO:{key}')
    return s

def add(parent,name,text):
    child=etree.SubElement(parent,NS(name));child.text=str(text);return child

def build_draft(payload):
    """An intentionally INCOMPLETE infNFe, for development review only.
    Will not invent CFOP/NCM/CRT/tax calculations or authorization.
    Provides a correctly namespaced scaffold but MUST fail full official XSD.
    """
    if payload.get('production') is True: raise DraftError('PRODUCAO_DESATIVADA')
    ide=payload.get('ide') or {}; emit=payload.get('emit') or {}; dest=payload.get('dest') or {}
    products=payload.get('items') or []
    if not isinstance(products,list) or not products: raise DraftError('ITENS_OBRIGATORIOS')
    cuf=numeric(ide,'cUF',2)
    if cuf!='33': raise DraftError('APENAS_UF_RJ')
    if numeric(ide,'tpAmb',1)!='2': raise DraftError('APENAS_HOMOLOGACAO')
    if numeric(ide,'mod',2)!='55': raise DraftError('APENAS_NFE_55')
    if not payload.get('tax_rules_approved'): raise DraftError('REGRAS_FISCAIS_NAO_APROVADAS')
    root=etree.Element(NS('NFe'),nsmap={None:NFE})
    inf=etree.SubElement(root,NS('infNFe'),versao='4.00')
    ide_el=etree.SubElement(inf,NS('ide'))
    for k in ['cUF','cNF','natOp','mod','serie','nNF','dhEmi','tpNF','idDest','cMunFG','tpImp','tpEmis','cDV','tpAmb','finNFe','indFinal','indPres','procEmi','verProc']:
        add(ide_el,k,required(ide,k))
    emit_el=etree.SubElement(inf,NS('emit'))
    add(emit_el,'CNPJ',numeric(emit,'CNPJ',14))
    add(emit_el,'xNome',required(emit,'xNome'))
    if not emit.get('CRT'): raise DraftError('REGIME_TRIBUTARIO_INDEFINIDO')
    # Deliberately not serializing fiscal tax group without a tax engine.
    # Name/address groups and totals remain blocked: the output is NOT official XML.
    add(emit_el,'CRT',numeric(emit,'CRT',1))
    de=etree.SubElement(inf,NS('dest'))
    if 'CNPJ' in dest: add(de,'CNPJ',numeric(dest,'CNPJ',14))
    elif 'CPF' in dest: add(de,'CPF',numeric(dest,'CPF',11))
    else: raise DraftError('DESTINATARIO_SEM_DOCUMENTO')
    add(de,'xNome',required(dest,'xNome'))
    for n,item in enumerate(products,1):
        node=etree.SubElement(inf,NS('det'),nItem=str(n)); prod=etree.SubElement(node,NS('prod'))
        for k in ['cProd','xProd','NCM','CFOP','uCom','qCom','vUnCom','vProd']:
            v=(numeric(item,k,8) if k=='NCM' else numeric(item,k,4) if k=='CFOP' else decimal(item,k) if k in ('qCom','vUnCom','vProd') else required(item,k))
            add(prod,k,v)
        # tax subgroup deliberately absent until fiscal rules approved/implemented
    return etree.tostring(root,encoding='UTF-8',xml_declaration=True)

def validate_external_xsd(xml_bytes,xsd_path):
    """Only an explicit independently sourced XSD counts as XSD validation."""
    parser=etree.XMLParser(resolve_entities=False,no_network=True,load_dtd=False)
    schema=etree.XMLSchema(etree.parse(str(xsd_path),parser))
    doc=etree.fromstring(xml_bytes,parser)
    if not schema.validate(doc): raise DraftError('XSD_REJEITADO: '+str(schema.error_log.last_error))
    return True
