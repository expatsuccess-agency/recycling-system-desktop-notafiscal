import tempfile
from pathlib import Path
import pytest
from tools.notafiscal.nfe_lab import make_test_case, load_xml, verify_local, NFE_NS
from lxml import etree

def test_signature_local_and_tamper():
    with tempfile.TemporaryDirectory() as d:
        assert 'E2_PROVA_LOCAL_OK' in make_test_case(Path(d))

def test_reject_xml_with_doctype():
    with tempfile.TemporaryDirectory() as d:
        p=Path(d)/'bad.xml'
        p.write_text('<!DOCTYPE NFe [<!ENTITY x "x">]><NFe>&x;</NFe>')
        with pytest.raises(ValueError):load_xml(p)

def test_reject_not_nfe():
    with tempfile.TemporaryDirectory() as d:
        p=Path(d)/'wrong.xml'
        p.write_bytes(etree.tostring(etree.Element('Other')))
        with pytest.raises(ValueError):verify_local(p)
