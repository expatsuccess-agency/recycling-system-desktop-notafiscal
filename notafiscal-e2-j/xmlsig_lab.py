#!/usr/bin/env python3
"""Offline XMLDSig interoperability laboratory. NOT an official NF-e emitter.
No SEFAZ communication. Does not check ICP-Brasil trust, authorization or fiscal validity.
"""
from datetime import datetime, timedelta, timezone
from pathlib import Path
from copy import deepcopy
import base64
import hashlib
import sys
from lxml import etree
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.x509.oid import NameOID

NS = "http://www.portalfiscal.inf.br/nfe"
DS = "http://www.w3.org/2000/09/xmldsig#"
C14N = "http://www.w3.org/TR/2001/REC-xml-c14n-20010315"
ENVELOPED = DS + "enveloped-signature"
RSA_SHA256 = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"
SHA256 = "http://www.w3.org/2001/04/xmlenc#sha256"


def b64(data): return base64.b64encode(data).decode("ascii")
def canonical(elem): return etree.tostring(elem, method="c14n", exclusive=False, with_comments=False)
def element(parent, name, text=None, **attrib):
    e = etree.SubElement(parent, "{%s}%s" % (DS, name), **attrib)
    if text is not None: e.text = text
    return e


def test_xml():
    root=etree.Element("{%s}NFe"%NS,nsmap={None:NS})
    inf=etree.SubElement(root,"{%s}infNFe"%NS,Id="NFe"+"1"*44,versao="4.00")
    # Deliberately minimal, NOT a valid Brazilian fiscal XML
    ide=etree.SubElement(inf,"{%s}ide"%NS)
    etree.SubElement(ide,"{%s}cUF"%NS).text="33"
    etree.SubElement(ide,"{%s}tpAmb"%NS).text="2"
    return root


def sign(root, key, cert):
    infs=root.xpath("./nfe:infNFe",namespaces={"nfe":NS})
    if len(infs)!=1: raise ValueError("exactly one infNFe required")
    inf=infs[0]
    ident=inf.get("Id","")
    if not ident.startswith("NFe") or len(ident)!=47: raise ValueError("invalid infNFe Id")
    if root.xpath("./ds:Signature",namespaces={"ds":DS}): raise ValueError("already signed")
    sig=etree.SubElement(root,"{%s}Signature"%DS,nsmap={"ds":DS})
    si=element(sig,"SignedInfo")
    element(si,"CanonicalizationMethod",Algorithm=C14N)
    element(si,"SignatureMethod",Algorithm=RSA_SHA256)
    ref=element(si,"Reference",URI="#"+ident)
    tr=element(ref,"Transforms")
    element(tr,"Transform",Algorithm=ENVELOPED)
    element(tr,"Transform",Algorithm=C14N)
    element(ref,"DigestMethod",Algorithm=SHA256)
    element(ref,"DigestValue",b64(hashlib.sha256(canonical(inf)).digest()))
    raw_sig=key.sign(canonical(si),padding.PKCS1v15(),hashes.SHA256())
    element(sig,"SignatureValue",b64(raw_sig))
    ki=element(sig,"KeyInfo")
    xdata=element(ki,"X509Data")
    element(xdata,"X509Certificate",b64(cert.public_bytes(serialization.Encoding.DER)))
    return root


def verify(root, expected_cert):
    sigs=root.xpath("./ds:Signature",namespaces={"ds":DS})
    infs=root.xpath("./nfe:infNFe",namespaces={"nfe":NS})
    if len(sigs)!=1 or len(infs)!=1: raise ValueError("missing/duplicate Signature or infNFe")
    sig,inf=sigs[0],infs[0]
    si=sig.find("{%s}SignedInfo"%DS)
    if si is None: raise ValueError("missing SignedInfo")
    cm=si.find("{%s}CanonicalizationMethod"%DS)
    sm=si.find("{%s}SignatureMethod"%DS)
    ref=si.find("{%s}Reference"%DS)
    if cm is None or cm.get("Algorithm")!=C14N or sm is None or sm.get("Algorithm")!=RSA_SHA256 or ref is None:
        raise ValueError("unsupported algorithms")
    if ref.get("URI")!="#"+inf.get("Id",""): raise ValueError("reference mismatch")
    transforms=ref.find("{%s}Transforms"%DS)
    if transforms is None or [x.get("Algorithm") for x in transforms]!=[ENVELOPED,C14N]:
        raise ValueError("unexpected transforms")
    digest_method=ref.find("{%s}DigestMethod"%DS)
    digest_value=ref.find("{%s}DigestValue"%DS)
    if digest_method is None or digest_method.get("Algorithm")!=SHA256 or digest_value is None:
        raise ValueError("wrong digest metadata")
    if b64(hashlib.sha256(canonical(inf)).digest())!=digest_value.text:
        raise ValueError("digest mismatch: signed data modified")
    value=sig.find("{%s}SignatureValue"%DS)
    embedded=sig.find("{%s}KeyInfo/{%s}X509Data/{%s}X509Certificate"%(DS,DS,DS))
    if value is None or embedded is None: raise ValueError("missing signature or certificate")
    if base64.b64decode(embedded.text)!=expected_cert.public_bytes(serialization.Encoding.DER):
        raise ValueError("embedded cert mismatch")
    expected_cert.public_key().verify(base64.b64decode(value.text),canonical(si),padding.PKCS1v15(),hashes.SHA256())
    return True


def main():
    key=rsa.generate_private_key(public_exponent=65537,key_size=2048)
    name=x509.Name([x509.NameAttribute(NameOID.COMMON_NAME,"LABORATORIO E2 - NAO VALIDO")])
    now=datetime.now(timezone.utc)
    cert=(x509.CertificateBuilder().subject_name(name).issuer_name(name).public_key(key.public_key())
          .serial_number(x509.random_serial_number()).not_valid_before(now-timedelta(days=1))
          .not_valid_after(now+timedelta(days=1)).sign(key,hashes.SHA256()))
    root=test_xml()
    sign(root,key,cert)
    verify(root,cert)
    print("E2_XMLDSIG_ASSINATURA_LOCAL=OK")
    print("E2_XMLDSIG_VERIFICACAO_LOCAL=OK")
    changed=deepcopy(root)
    changed.find("{%s}infNFe/{%s}ide/{%s}cUF"%(NS,NS,NS)).text="35"
    try:
        verify(changed,cert)
    except ValueError as ex:
        if "digest mismatch" not in str(ex): raise
        print("E2_XMLDSIG_ALTERACAO_DETECTADA=OK")
    else:
        raise AssertionError("Tampering undetected")
    print("STATUS=LABORATORIO_SEM_VALIDADE_FISCAL")
    print("NENHUM_ENVIO_SEFAZ=SIM")

if __name__=="__main__": main()
