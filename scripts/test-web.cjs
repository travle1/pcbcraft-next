// Original browser acceptance check against the built static bundle, under a Pages-style prefix.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const { chromium } = require('../work/web-test/node_modules/playwright');
const root = path.resolve(__dirname, '..');
const dist = path.join(root, 'target/web-dist');
const proof = path.join(root, 'work/proof');
const prefix = '/pcbcraft-next/';
const server = http.createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    if (!pathname.startsWith(prefix)) { response.writeHead(404).end(); return; }
    let file = path.resolve(dist, pathname.slice(prefix.length));
    if (file !== dist && !file.startsWith(dist + path.sep)) { response.writeHead(403).end(); return; }
    if (file === dist || pathname.endsWith('/')) file = path.join(file, 'index.html');
    const types = { '.wasm': 'application/wasm', '.js': 'text/javascript', '.html': 'text/html', '.json': 'application/json' };
    fs.stat(file, (error, stat) => {
        if (error || !stat.isFile()) { response.writeHead(404).end(); return; }
        response.writeHead(200, { 'Content-Type': types[path.extname(file)] || 'text/plain' });
        fs.createReadStream(file).pipe(response);
    });
});

(async () => {
    await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
    const executablePath = process.env.PCBCRAFT_CHROMIUM_PATH;
    const browser = await chromium.launch({ executablePath, headless: true,
        args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
    try {
        const context = await browser.newContext({ viewport: { width: 1600, height: 1000 },
            permissions: ['clipboard-read', 'clipboard-write'], acceptDownloads: true });
        const page = await context.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        const base = `http://127.0.0.1:${server.address().port}${prefix}`;
        await page.goto(base + '?demo=1');
        await page.waitForFunction(() => document.documentElement.dataset.pcbcraft === 'ready', {}, { timeout: 60000 });
        await page.waitForTimeout(800);
        await page.screenshot({ path: path.join(proof, 'web-schematic.png') });
        const settle = () => page.waitForTimeout(180);
        const click = async (target, x, y) => {
            await target.mouse.move(x, y);
            await target.waitForTimeout(70);
            await target.mouse.down();
            await target.waitForTimeout(70);
            await target.mouse.up();
            await target.waitForTimeout(100);
        };
        const download = async () => {
            const pending = page.waitForEvent('download');
            await click(page, 188, 16);
            const result = await pending;
            assert.ok(result.suggestedFilename().endsWith('.pc'));
            const file = await result.path();
            await settle();
            return JSON.parse(fs.readFileSync(file, 'utf8'));
        };
        const original = await download();
        console.log('Initial download passed');
        assert.equal(original.format, 'pcbcraft-project-draft/7');
        assert.ok(original.project.main.symbols.length > 20);
        // Use the real Properties field and shared Session history.
        await page.mouse.move(1490, 326);
        await settle();
        await click(page, 1490, 326);
        await settle();
        await page.keyboard.press('Control+a');
        await page.keyboard.type('Browser edited MCU', {delay: 35});
        await page.keyboard.press('Enter');
        await settle();
        await click(page, 1400, 463);
        await settle();
        await page.screenshot({path: path.join(proof, 'web-edit.png')});
        await click(page, 1200, 900);
        await settle();
        const edited = await download();
        console.log('Edited download received');
        assert.notDeepEqual(edited, original);
        assert.ok(edited.project.main.components.some(component => component.value === 'Browser edited MCU'));
        await click(page, 210, 16);
        await settle();
        assert.deepEqual(await download(), original);
        await page.mouse.move(232, 16);
        await settle();
        await click(page, 232, 16);
        await settle();
        assert.deepEqual(await download(), edited);
        fs.writeFileSync(path.join(proof, 'web-roundtrip.pc'), JSON.stringify(edited));
        let unloadWarning = false;
        page.once('dialog', async dialog => {
            assert.equal(dialog.type(), 'beforeunload');
            unloadWarning = true;
            await dialog.dismiss();
        });
        await page.close({ runBeforeUnload: true });
        await settle();
        assert.ok(unloadWarning && !page.isClosed(), 'Downloads must retain the unsaved-tab warning');
        // Browser-specific Assistant: real data copying, with no inert local-agent command.
        await click(page, 1450, 945);
        await settle();
        await click(page, 1280, 832);
        await click(page, 1280, 925);
        const copied = await page.evaluate(() => navigator.clipboard.readText());
        assert.ok(copied.includes('Browser edited MCU') && copied.includes('"live_mcp":false'));
        await page.screenshot({ path: path.join(proof, 'web-assistant.png') });
        // A second instance imports the browser-generated bytes through the real file picker.
        const reopened = await context.newPage();
        reopened.on('pageerror', error => errors.push(String(error)));
        await reopened.goto(base);
        await reopened.waitForFunction(() => document.documentElement.dataset.pcbcraft === 'ready');
        await reopened.waitForTimeout(500);
        await click(reopened, 900, 500);
        await reopened.waitForTimeout(100);
        const choose = async (payload) => {
            const pending = reopened.waitForEvent('filechooser');
            await reopened.keyboard.down('Control');
            await reopened.keyboard.down('o');
            await reopened.waitForTimeout(80);
            await reopened.keyboard.up('o');
            await reopened.keyboard.up('Control');
            await (await pending).setFiles(payload);
            await reopened.waitForTimeout(350);
        };
        await choose({ name: 'roundtrip.pc', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(edited)) });
        const exportReopened = async () => {
            const pending = reopened.waitForEvent('download');
            await click(reopened, 188, 16);
            const file = await (await pending).path();
            await reopened.waitForTimeout(180);
            return JSON.parse(fs.readFileSync(file, 'utf8'));
        };
        assert.deepEqual(await exportReopened(), edited);
        await choose([]);
        assert.deepEqual(await exportReopened(), edited, 'Cancelled import must preserve the live project');
        await choose({ name: 'invalid.pc', mimeType: 'application/json', buffer: Buffer.from('{"format":"unknown"}') });
        assert.deepEqual(await exportReopened(), edited, 'Invalid import must preserve the live project');
        await choose({ name: 'too-large.pc', mimeType: 'application/json', buffer: Buffer.alloc(4 * 1024 * 1024 + 1) });
        assert.deepEqual(await exportReopened(), edited, 'Oversized import must preserve the live project');
        await reopened.setViewportSize({ width: 1000, height: 700 });
        await reopened.waitForTimeout(500);
        await click(reopened, 900, 500);
        await reopened.waitForTimeout(100);
        await reopened.screenshot({ path: path.join(proof, 'web-import-error.png') });
        await click(reopened, 690, 468);
        await reopened.keyboard.down('f');
        await reopened.waitForTimeout(80);
        await reopened.keyboard.up('f');
        await reopened.waitForTimeout(350);
        await reopened.screenshot({ path: path.join(proof, 'web-compact.png') });
        await click(reopened, 860, 645);
        await reopened.screenshot({ path: path.join(proof, 'web-assistant-compact.png') });
        assert.deepEqual(errors, []);
        console.log(JSON.stringify({ browser: browser.version(), errors, base,
            status: 'render/edit/undo/redo/download/reimport/cancel/invalid/oversized/unload/clipboard/full/compact passed' }));
        await context.close();
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; }).finally(() => server.close());
