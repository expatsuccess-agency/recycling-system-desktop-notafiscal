import unittest,tempfile
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import xmlsig_lab
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography import x509
from cryptography.hazmat.primitives import hashes
from cryptography.x509.oid import NameOID
from datetime import datetime,timedelta,timezone
from copy import deepcopy

def certpair():
    k=rsa.generate_private_key(public_exponent=65537,key_size=2048)
    n=x509.Name([x509.NameAttribute(NameOID.COMMON_NAME,'TESTE SEM VALIDADE FISCAL')]);now=datetime.now(timezone.utc)
    c=(x509.CertificateBuilder().subject_name(n).issuer_name(n).public_key(k.public_key()).serial_number(x509.random_serial_number())
       .not_valid_before(now-timedelta(days=1)).not_valid_after(now+timedelta(days=1)).sign(k,hashes.SHA256()))
    return k,c
class E2JTests(unittest.TestCase):
    def test_sign_verify(self):
        k,c=certpair();r=xmlsig_lab.test_xml();xmlsig_lab.sign(r,k,c);self.assertTrue(xmlsig_lab.verify(r,c))
    def test_tampering(self):
        k,c=certpair();r=xmlsig_lab.test_xml();xmlsig_lab.sign(r,k,c)
        r.find('{%s}infNFe/{%s}ide/{%s}cUF'%(xmlsig_lab.NS,xmlsig_lab.NS,xmlsig_lab.NS)).text='35'
        with self.assertRaisesRegex(ValueError,'digest mismatch'):xmlsig_lab.verify(r,c)
    def test_non_nfe_xml_rejected(self):
        from lxml import etree
        k,c=certpair();r=etree.Element('foo')
        with self.assertRaises(ValueError):xmlsig_lab.sign(r,k,c)
    def test_gate_unapproved(self):
        import gate
        self.assertIn('J',gate.REQUIREMENTS)
        self.assertTrue(all(len(v)>0 for v in gate.REQUIREMENTS.values()))
    def test_duplicate_signature_rejected(self):
        k,c=certpair();r=xmlsig_lab.test_xml();xmlsig_lab.sign(r,k,c)
        with self.assertRaisesRegex(ValueError,'already signed'):xmlsig_lab.sign(r,k,c)
if __name__=='__main__':unittest.main()
