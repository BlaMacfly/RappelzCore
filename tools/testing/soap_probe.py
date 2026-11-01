"""Independent HTTP/SOAP acceptance probes; credentials stay in caller memory."""
import base64
import http.client
import xml.etree.ElementTree as ET

SOAP = 'http://schemas.xmlsoap.org/soap/envelope/'


def envelope(command, namespace='urn:AC'):
    root = ET.Element('{'+SOAP+'}Envelope')
    body = ET.SubElement(root, '{'+SOAP+'}Body')
    method = ET.SubElement(body, '{'+namespace+'}executeCommand')
    ET.SubElement(method, 'command').text = command
    return ET.tostring(root, encoding='utf-8', xml_declaration=True)


def request(host, port, credentials=None, command='server info', namespace='urn:AC', raw=None, chunked=False, method='POST'):
    connection = http.client.HTTPConnection(host, port, timeout=20)
    headers = {'Content-Type': 'text/xml; charset=utf-8', 'SOAPAction': '"'+namespace+'#executeCommand"'}
    if credentials:
        headers['Authorization'] = 'Basic '+base64.b64encode(':'.join(credentials).encode()).decode('ascii')
    data = envelope(command, namespace) if raw is None else raw
    if chunked:
        data = [data[i:i+7] for i in range(0, len(data), 7)]
    try:
        connection.request(method, '/', body=data, headers=headers, encode_chunked=chunked)
        response = connection.getresponse(); body = response.read()
        xml = ET.fromstring(body)
        result = xml.find('.//result')
        fault = xml.find('.//{'+SOAP+'}Fault/faultstring')
        return dict(status=response.status, result=result.text if result is not None else None,
                    fault=fault.text if fault is not None else None, challenge=response.getheader('WWW-Authenticate'),
                    namespace=next((n.tag.split('}')[0][1:] for n in xml.iter() if n.tag.endswith('}executeCommandResponse')), None))
    finally:
        connection.close()


def probe(host, port, admin, low, other_realm):
    checks = []
    def check(name, response, status, contains=None):
        passed = response['status'] == status
        passed = passed and (contains in (response['result'] or '') if contains else bool(response['fault']))
        checks.append(dict(name=name, passed=passed, response=response))
    check('missing credentials return HTTP 401 and Basic challenge', request(host, port), 401)
    checks[-1]['passed'] &= bool(checks[-1]['response']['challenge'])
    check('wrong password cannot execute a command', request(host, port, (admin[0], 'wrong-pass')), 401)
    check('ordinary player cannot execute a command', request(host, port, low), 403)
    check('administrator of another realm is denied', request(host, port, other_realm), 403)
    for namespace in ['urn:AC', 'urn:TC']:
        r = request(host, port, admin, namespace=namespace)
        check(namespace+' authenticated executeCommand returns live server info', r, 200, 'joueurs : 2')
        checks[-1]['passed'] &= r['namespace'] == namespace
    check('HTTP chunked RPC body reaches the same dispatcher', request(host, port, admin, chunked=True), 200, 'RappelzCore')
    check('unknown console command becomes a SOAP Fault', request(host, port, admin, command='unknown_command'), 500)
    check('malformed XML becomes a SOAP Fault', request(host, port, admin, raw=b'<broken'), 500)
    check('DTD and entity expansion are refused', request(host, port, admin, raw=b'<!DOCTYPE foo [<!ENTITY x "exit">]>'+envelope('&x;')), 500)
    check('duplicate operations cannot execute shutdown', request(host, port, admin, raw=envelope('server info').replace(b'</ns0:Body>', b'<ns1:executeCommand><command>exit</command></ns1:executeCommand></ns0:Body>')), 500)
    check('body size limit returns HTTP 413', request(host, port, admin, raw=b'x'*17000), 413)
    check('GET cannot execute a command', request(host, port, admin, method='GET'), 405)
    check('server stays healthy after rejected requests', request(host, port, admin), 200, 'joueurs : 2')
    notice = 'SOAP validation <raw> & synchronisation'
    check('authenticated announcement accepts XML escaping', request(host, port, admin, command='announce '+notice), 200, 'Annonce envoyée')
    return dict(success=all(c['passed'] for c in checks), checks=checks, notice=notice)
