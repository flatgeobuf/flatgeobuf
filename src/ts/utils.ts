export function toUint8Array(bb: flatbuffers.ByteBuffer) {
    return new Uint8Array(bb.bytes().buffer, bb.position())
}

export function toInt32(num: number) {
    const arr = new ArrayBuffer(4)
    const view = new DataView(arr)
    view.setUint32(0, num, false)
    return new Uint8Array(arr)
}

export function getInt32(bytes: Uint8Array) {
    return new DataView(bytes.buffer).getInt32(0)
}
