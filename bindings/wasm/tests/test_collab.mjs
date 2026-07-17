import test from 'node:test';
import assert from 'node:assert'
import { Model } from "../pkg/wasm.js";

// Two models exchanging frames directly (no websocket): the same byte
// protocol the relay server speaks.

function connect(a, b) {
    // Both sides start their handshake; pump frames until quiet.
    let toB = [a.collabStartSync()];
    let toA = [b.collabStartSync()];
    let guard = 0;
    while (toA.length > 0 || toB.length > 0) {
        assert.ok(guard++ < 100, "handshake does not quiesce");
        const frameForB = toB.shift();
        if (frameForB !== undefined && frameForB.length > 0) {
            const outcome = b.collabHandleFrame(frameForB);
            if (outcome.replies.length > 0) toA.push(outcome.replies);
        }
        const frameForA = toA.shift();
        if (frameForA !== undefined && frameForA.length > 0) {
            const outcome = a.collabHandleFrame(frameForA);
            if (outcome.replies.length > 0) toB.push(outcome.replies);
        }
    }
}

function deliver(from, to) {
    const frame = from.collabFlushLocal();
    if (frame !== undefined) {
        return to.collabHandleFrame(frame);
    }
    return undefined;
}

test('collaboration: edits converge and formulas evaluate', () => {
    const a = new Model('Workbook1', 'en', 'UTC', 'en');
    const b = new Model('Workbook1', 'en', 'UTC', 'en');
    a.collabAttach(1);
    b.collabAttach(2);
    assert.ok(a.collabIsAttached());
    connect(a, b);

    a.setUserInput(0, 1, 1, "21");
    const outcome = deliver(a, b);
    assert.ok(outcome.appliedUpdate);
    assert.strictEqual(b.getFormattedCellValue(0, 1, 1), "21");

    b.setUserInput(0, 1, 2, "=A1*2");
    deliver(b, a);
    assert.strictEqual(a.getFormattedCellValue(0, 1, 2), "42");

    // Nothing pending → no frame.
    assert.strictEqual(a.collabFlushLocal(), undefined);
});

test('collaboration: presence round trip', () => {
    const a = new Model('Workbook1', 'en', 'UTC', 'en');
    const b = new Model('Workbook1', 'en', 'UTC', 'en');
    a.collabAttach(7);
    b.collabAttach(8);
    connect(a, b);

    const frame = a.collabSetPresence('{"name":"ana","cell":"A1"}');
    const outcome = b.collabHandleFrame(frame);
    assert.ok(outcome.presenceChanged);
    const presence = b.collabPresence();
    assert.deepStrictEqual(presence, [
        { clientId: 7, state: '{"name":"ana","cell":"A1"}' },
    ]);

    const goodbye = a.collabClearPresence();
    b.collabHandleFrame(goodbye);
    assert.deepStrictEqual(b.collabPresence(), []);
});

test('collaboration: attach twice fails, unattached calls fail', () => {
    const a = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.throws(() => a.collabStartSync());
    a.collabAttach(1);
    assert.throws(() => a.collabAttach(2));
});
