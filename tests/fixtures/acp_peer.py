"""Original deterministic ACP v1 protocol fixture; no model, credentials or external code."""
import json
import os
import pathlib
import select
import subprocess
import sys

mode = sys.argv[1] if len(sys.argv) > 1 else 'reply'
record = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else None


def receive():
    line = sys.stdin.readline()
    if not line:
        raise EOFError('client closed')
    return json.loads(line)


def send(value):
    print(json.dumps(value), flush=True)


def reply(request, result):
    send({'jsonrpc': '2.0', 'id': request['id'], 'result': result})


def update(kind, **fields):
    send({'jsonrpc': '2.0', 'method': 'session/update', 'params': {
        'sessionId': 'fixture-session', 'update': {'sessionUpdate': kind, **fields}}})


try:
    initial = receive()
    assert initial['method'] == 'initialize'
    assert initial['params']['protocolVersion'] == 1
    assert initial['params']['clientCapabilities']['fs'] == {'readTextFile': False, 'writeTextFile': False}
    assert not initial['params']['clientCapabilities']['terminal']
    reply(initial, {'protocolVersion': 2 if mode == 'wrong-version' else 1,
                    'agentCapabilities': {}, 'agentInfo': {'name': 'Scripted ACP fixture', 'version': '1'}})
    create = receive()
    assert create['method'] == 'session/new'
    assert pathlib.Path(create['params']['cwd']).is_absolute()
    assert list(pathlib.Path(create['params']['cwd']).iterdir()) == []
    server, = create['params']['mcpServers']
    assert server['name'] == 'pcbcraft-app' and not server['env']
    assert pathlib.Path(server['command']).is_absolute()
    reply(create, {'sessionId': 'fixture-session'})
    prompt = receive()
    assert prompt['method'] == 'session/prompt'
    text = prompt['params']['prompt'][0]['text']
    assert 'Never inspect KiCad implementation' in text
    data = json.loads(text[text.index('\n{"turn"') + 1:])
    assert data['messages'][-1]['role'] == 'user'
    if record:
        record.write_text(json.dumps({'pid': os.getpid(), 'request': data}))
    if mode == 'oversized':
        sys.stdout.write('x' * (512 * 1024 + 1) + '\n')
        sys.stdout.flush()
    elif mode == 'hang':
        update('agent_message_chunk', content={'type': 'text', 'text': 'I am inspecting the current sheet.'})
        cancelled = receive()
        assert cancelled['method'] == 'session/cancel'
        if record:
            record.write_text(json.dumps({'cancelled': True}))
    else:
        # Every permission is a fresh one-shot decision. No file/terminal approval.
        for index, (title, allowed) in enumerate([
            ('mcp__pcbcraft-app__inspect_project', True),
            ('mcp__pcbcraft-app__prepare_edit', True),
            ('mcp__pcbcraft-app__apply_edit', False),
            ('mcp__pcbcraft-app__inspect_project extra', False),
            ('Read local file', False),
        ]):
            send({'jsonrpc': '2.0', 'id': f'permission-{index}', 'method': 'session/request_permission', 'params': {
                'sessionId': 'fixture-session', 'toolCall': {'title': title, 'kind': 'other'},
                'options': [{'optionId': 'yes', 'kind': 'allow_once'}, {'optionId': 'no', 'kind': 'reject_once'}]}})
            answer = receive()
            assert answer['result']['outcome']['optionId'] == ('yes' if allowed else 'no')
        send({'jsonrpc': '2.0', 'id': 'filesystem', 'method': 'fs/read_text_file', 'params': {}})
        assert receive()['error']['code'] == -32601
        update('agent_thought_chunk', content={'type': 'text', 'text': 'Hidden reasoning'})
        send({'jsonrpc': '2.0', 'method': 'session/update', 'params': {
            'sessionId': 'wrong-session', 'update': {'sessionUpdate': 'agent_message_chunk',
                                                  'content': {'type': 'text', 'text': 'Wrong session'}}}})
        update('agent_message_chunk', content={'type': 'text', 'text': 'I can inspect '})
        if mode == 'proposal':
            client = subprocess.Popen([server['command'], *server['args']], stdin=subprocess.PIPE,
                                      stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
            sequence = 0

            def rpc(method, params):
                global sequence
                sequence += 1
                client.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': sequence, 'method': method, 'params': params}) + '\n')
                client.stdin.flush()
                assert select.select([client.stdout], [], [], 5)[0], 'MCP fixture timed out'
                response = json.loads(client.stdout.readline())
                assert 'error' not in response, response
                return response['result']

            def tool(name, arguments):
                result = rpc('tools/call', {'name': name, 'arguments': arguments})
                assert not result.get('isError'), result
                return result.get('structuredContent') or json.loads(result['content'][0]['text'])

            try:
                rpc('initialize', {'protocolVersion': '2025-06-18', 'capabilities': {},
                                   'clientInfo': {'name': 'original-acp-fixture', 'version': '1'}})
                client.stdin.write(json.dumps({'jsonrpc': '2.0', 'method': 'notifications/initialized'}) + '\n')
                client.stdin.flush()
                project = tool('inspect_project', {})
                tool('pcb_reference', {})
                tool('list_parts', {})
                tool('inspect_sheet', {'sheet': project['active_sheet']})
                plan = tool('prepare_edit', {'expected': project['snapshot'], 'sheet': project['active_sheet'],
                    'summary': 'Add two connected resistors', 'operations': [
                        {'op': 'place', 'key': 'a', 'kind': 'resistor', 'at': {'x': 0, 'y': 0}},
                        {'op': 'place', 'key': 'b', 'kind': 'resistor', 'at': {'x': 50800000, 'y': 0}},
                        {'op': 'connect', 'from': {'symbol': {'local': 'a'}, 'terminal': '2'},
                         'to': {'symbol': {'local': 'b'}, 'terminal': '1'}}]})
                assert plan['base'] == project['snapshot']
                assert tool('inspect_project', {})['snapshot'] == project['snapshot']
            finally:
                client.stdin.close()
                try:
                    client.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    client.kill()
                    client.wait(timeout=5)
            update('agent_message_chunk', content={'type': 'text', 'text': 'this sheet. Two connected resistors are ready for review; the design has not changed.'})
            reply(prompt, {'stopReason': 'end_turn'})
        elif mode == 'error':
            send({'jsonrpc': '2.0', 'id': prompt['id'], 'error': {'code': -32000, 'message': 'Fixture authentication required'}})
        else:
            update('agent_message_chunk', content={'type': 'text', 'text': 'this sheet.'})
            reply(prompt, {'stopReason': 'end_turn'})
except EOFError:
    pass
