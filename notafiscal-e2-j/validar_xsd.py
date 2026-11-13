#!/usr/bin/env python3
"""Validação XSD OFFLINE. Requer esquema oficial fornecido e sua árvore de includes.
Não cria Nota Fiscal nem valida legislação tributária.
"""
from pathlib import Path
from lxml import etree
import argparse,sys
p=argparse.ArgumentParser()
p.add_argument('--xml',required=True,type=Path)
p.add_argument('--xsd',required=True,type=Path)
a=p.parse_args()
if not a.xml.is_file() or not a.xsd.is_file(): p.error('Arquivo XML/XSD ausente')
try:
    parser=etree.XMLParser(resolve_entities=False, no_network=True,load_dtd=False, huge_tree=False)
    schema_doc=etree.parse(str(a.xsd),parser)
    schema=etree.XMLSchema(schema_doc)
    xml=etree.parse(str(a.xml),parser)
    if schema.validate(xml):
        print('XSD_VALIDACAO=OK (somente esquema informado)')
    else:
        for e in list(schema.error_log)[:8]:print(f'XSD_ERRO: linha {e.line}: {e.message}',file=sys.stderr)
        sys.exit(2)
except (etree.XMLSyntaxError,etree.XMLSchemaParseError,OSError) as e:
    print(f'XSD_ERRO={type(e).__name__}: {e}',file=sys.stderr);sys.exit(2)
