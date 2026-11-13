#!/usr/bin/env python3
"""Ferramentas offline de engenharia de NF-e. NÃO são um emissor fiscal.

Assinatura XMLDSig de laboratório: RSA-SHA256, C14N 1.0, enveloped.
Sem checagem de cadeia ICP-Brasil/revogação/identidade da empresa,
sem autorização SEFAZ, sem garantia de aceitação no perfil NF-e em vigor.
"""
import argparse
import base64
from datetime import datetime, timedelta, timezone
from getpass import getpass
import hashlib
import json
from pathlib import Path
import sys
import tempfile
from lxml import etree
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa
from cryptography.hazmat.primitives.serialization import pkcs12
from cryptography.x509.oid import NameOID

NFE_NS = "http://www.portalfiscal.inf.br/nfe"
DS_NS = "http://www.w3.org/2000/09/xmldsig#"
C14N = "http://www.w3.org/TR/2001/REC-xml-c14n-20010315"
DIGEST = DS_NS + "sha256"
SIGNATURE = DS_NS + "rsa-sha256"
ENVELOPED = DS_NS + "enveloped-signature"
MAX_XML = 5_000_000


def load_xml(path):
    p = Path(path)
    if not p.is_file() or p.stat().st_size > MAX_XML:
        raise ValueError("Arquivo XML ausente ou maior que 5 MB")
    parser = etree.XMLParser(resolve_entities=False, no_network=True, load_dtd=False, huge_tree=False,
                             remove_blank_text=False)
    root = etree.fromstring(p.read_bytes(), parser=parser)
    if root.getroottree().docinfo.doctype:
        raise ValueError("DOCTYPE não permitido")
    return root


def infnfe(root):
    if root.tag != f"{{{NFE_NS}}}NFe":
        raise ValueError("A raiz deve ser NFe do namespace oficial")
    matches = root.xpath("./n:infNFe", namespaces={"n": NFE_NS})
    if len(matches) != 1:
        raise ValueError("É necessário exatamente um infNFe")
    info = matches[0]
    ident = info.get("Id")
    if not ident or not ident.startswith("NFe") or len(ident) != 47 or not ident[3:].isdigit():
        raise ValueError("Id infNFe deve conter NFe + 44 dígitos")
    if len(root.xpath("//*[@Id=$ident]", ident=ident)) != 1:
        raise ValueError("Id duplicado: assinatura ambígua")
    return info, ident


def canon(element):
    return etree.tostring(element, method="c14n", exclusive=False, with_comments=False)


def signed_info(parent, ident, digest_b64):
    si = etree.SubElement(parent, f"{{{DS_NS}}}SignedInfo")
    etree.SubElement(si, f"{{{DS_NS}}}CanonicalizationMethod", Algorithm=C14N)
    etree.SubElement(si, f"{{{DS_NS}}}SignatureMethod", Algorithm=SIGNATURE)
    reference = etree.SubElement(si, f"{{{DS_NS}}}Reference", URI="#" + ident)
    transforms = etree.SubElement(reference, f"{{{DS_NS}}}Transforms")
    etree.SubElement(transforms, f"{{{DS_NS}}}Transform", Algorithm=ENVELOPED)
    etree.SubElement(transforms, f"{{{DS_NS}}}Transform", Algorithm=C14N)
    etree.SubElement(reference, f"{{{DS_NS}}}DigestMethod", Algorithm=DIGEST)
    etree.SubElement(reference, f"{{{DS_NS}}}DigestValue").text = digest_b64
    return si


def sign_xml(xml, pfx, output, password):
    root = load_xml(xml)
    info, ident = infnfe(root)
    if root.find(f"{{{DS_NS}}}Signature") is not None:
        raise ValueError("Documento já assinado; não sobrescrever")
    key, certificate, _ = pkcs12.load_key_and_certificates(Path(pfx).read_bytes(), password.encode())
    if certificate is None or not isinstance(key, rsa.RSAPrivateKey):
        raise ValueError("Certificado A1 RSA com chave privada necessário")
    now = datetime.now(timezone.utc)
    if not certificate.not_valid_before_utc <= now <= certificate.not_valid_after_utc:
        raise ValueError("Certificado fora da vigência")
    digest = base64.b64encode(hashlib.sha256(canon(info)).digest()).decode()
    sig = etree.SubElement(root, f"{{{DS_NS}}}Signature", nsmap={"ds": DS_NS})
    si = signed_info(sig, ident, digest)
    value = key.sign(canon(si), padding.PKCS1v15(), hashes.SHA256())
    etree.SubElement(sig, f"{{{DS_NS}}}SignatureValue").text = base64.b64encode(value).decode()
    ki = etree.SubElement(sig, f"{{{DS_NS}}}KeyInfo")
    xdata = etree.SubElement(ki, f"{{{DS_NS}}}X509Data")
    etree.SubElement(xdata, f"{{{DS_NS}}}X509Certificate").text = base64.b64encode(
        certificate.public_bytes(serialization.Encoding.DER)).decode()
    output = Path(output)
    if output.exists():
        raise ValueError("Arquivo de saída já existe: não sobrescrever")
    output.write_bytes(etree.tostring(root, encoding="UTF-8", xml_declaration=True))
    return output


def verify_local(xml):
    root = load_xml(xml)
    info, ident = infnfe(root)
    sigs = root.findall(f"{{{DS_NS}}}Signature")
    if len(sigs) != 1:
        raise ValueError("Exatamente uma Signature como filha de NFe é exigida")
    sig = sigs[0]
    si = sig.find(f"{{{DS_NS}}}SignedInfo")
    if si is None:
        raise ValueError("SignedInfo ausente")
    ref = si.find(f"{{{DS_NS}}}Reference")
    if ref is None or ref.get("URI") != "#" + ident:
        raise ValueError("Reference diferente de infNFe")
    method = si.find(f"{{{DS_NS}}}SignatureMethod")
    canon_method = si.find(f"{{{DS_NS}}}CanonicalizationMethod")
    if method is None or method.get("Algorithm") != SIGNATURE or canon_method is None or canon_method.get("Algorithm") != C14N:
        raise ValueError("Algoritmo inesperado")
    transforms = ref.find(f"{{{DS_NS}}}Transforms")
    if transforms is None or [t.get("Algorithm") for t in transforms] != [ENVELOPED, C14N]:
        raise ValueError("Transforms inesperados")
    digest_method = ref.find(f"{{{DS_NS}}}DigestMethod")
    if digest_method is None or digest_method.get("Algorithm") != DIGEST:
        raise ValueError("DigestMethod inesperado")
    expected = base64.b64encode(hashlib.sha256(canon(info)).digest()).decode()
    actual = ref.findtext(f"{{{DS_NS}}}DigestValue")
    if expected != actual:
        raise ValueError("Digest divergente: XML alterado")
    der_b64 = sig.findtext(f"{{{DS_NS}}}KeyInfo/{{{DS_NS}}}X509Data/{{{DS_NS}}}X509Certificate")
    signature_b64 = sig.findtext(f"{{{DS_NS}}}SignatureValue")
    if not der_b64 or not signature_b64:
        raise ValueError("Certificado ou assinatura ausente")
    cert = x509.load_der_x509_certificate(base64.b64decode(der_b64, validate=True))
    cert.public_key().verify(base64.b64decode(signature_b64, validate=True), canon(si), padding.PKCS1v15(), hashes.SHA256())
    return {"assinatura_criptografica_local": True, "nfe_autorizada": False,
            "validacao_icp_brasil": False, "validacao_xsd": False,
            "certificado_sha256": cert.fingerprint(hashes.SHA256()).hex(), "referencia": "#" + ident}


def xsd_validation(xml, xsd):
    # Apenas schemas oficiais obtidos e versionados separadamente; acesso à rede desabilitado.
    parser = etree.XMLParser(resolve_entities=False, no_network=True, load_dtd=False)
    schema = etree.XMLSchema(etree.parse(str(xsd), parser=parser))
    root = load_xml(xml)
    schema.assertValid(root)
    return True


def make_test_case(tmp):
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    subject = issuer = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "TESTE LOCAL SEM VALIDADE FISCAL")])
    now = datetime.now(timezone.utc)
    cert = (x509.CertificateBuilder().subject_name(subject).issuer_name(issuer).public_key(key.public_key())
            .serial_number(x509.random_serial_number()).not_valid_before(now-timedelta(days=1))
            .not_valid_after(now+timedelta(days=1)).sign(key, hashes.SHA256()))
    pfx = tmp/'teste.pfx'
    pfx.write_bytes(pkcs12.serialize_key_and_certificates(b"teste", key, cert, None,
        serialization.BestAvailableEncryption(b"SenhaTeste123!")))
    xml = tmp/'teste.xml'
    ident = 'NFe'+'1'*44
    root = etree.Element(f"{{{NFE_NS}}}NFe", nsmap={None:NFE_NS})
    info = etree.SubElement(root, f"{{{NFE_NS}}}infNFe", Id=ident, versao="4.00")
    etree.SubElement(info, f"{{{NFE_NS}}}ide")
    xml.write_bytes(etree.tostring(root, encoding='UTF-8', xml_declaration=True))
    signed = tmp/'signed.xml'
    sign_xml(xml,pfx,signed,'SenhaTeste123!')
    verify_local(signed)
    changed = tmp/'changed.xml'
    changed.write_bytes(signed.read_bytes().replace(b'versao="4.00"',b'versao="4.01"'))
    try:
        verify_local(changed)
    except ValueError as exc:
        assert 'Digest divergente' in str(exc)
    else:
        raise AssertionError('Assinatura não detectou alteração')
    return 'E2_PROVA_LOCAL_OK (NÃO é conformidade NF-e nem homologação SEFAZ)'


def create_manifest(xml, out):
    xml_path = Path(xml)
    data = xml_path.read_bytes()
    if not data or len(data)>MAX_XML:
        raise ValueError('Arquivo inválido')
    target = Path(out)
    if target.exists():
        raise ValueError('Manifesto já existe')
    target.write_text(json.dumps({"categoria": "ARTEFATO_INTERNO_NAO_FISCAL",
      "arquivo": xml_path.name, "sha256": hashlib.sha256(data).hexdigest(),
      "autoridade_sefaz_confirmada": False, "danfe": False,
      "criado_utc": datetime.now(timezone.utc).isoformat()},indent=2,ensure_ascii=False)+"\n")
    return target


def main():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest='cmd',required=True)
    sub.add_parser('selftest')
    a = sub.add_parser('sign-lab');a.add_argument('--xml',required=True);a.add_argument('--pfx',required=True);a.add_argument('--out',required=True)
    a.add_argument('--ack-test-only', action='store_true')
    a = sub.add_parser('verify-local');a.add_argument('--xml',required=True)
    a = sub.add_parser('validate-xsd');a.add_argument('--xml',required=True);a.add_argument('--xsd',required=True)
    a = sub.add_parser('manifest');a.add_argument('--xml',required=True);a.add_argument('--out',required=True)
    args = p.parse_args()
    if args.cmd == 'selftest':
        with tempfile.TemporaryDirectory(prefix='nfe-e2-') as d:
            print(make_test_case(Path(d)))
    elif args.cmd == 'sign-lab':
        if not args.ack_test_only: raise ValueError('Use --ack-test-only para confirmar uso exclusivo em testes')
        secret = getpass('Senha A1 (não armazenada): ')
        print('Assinado localmente:',sign_xml(args.xml,args.pfx,args.out,secret))
        print('NÃO autorizado / NÃO homologado / NÃO transmitir como NF-e')
    elif args.cmd == 'verify-local': print(json.dumps(verify_local(args.xml),indent=2))
    elif args.cmd == 'validate-xsd':print('XSD local OK:',xsd_validation(args.xml,args.xsd))
    elif args.cmd == 'manifest':print('Manifesto interno criado:',create_manifest(args.xml,args.out))

if __name__ == '__main__':
    try: main()
    except Exception as exc:
        print(f'ERRO: {exc}', file=sys.stderr)
        sys.exit(1)
