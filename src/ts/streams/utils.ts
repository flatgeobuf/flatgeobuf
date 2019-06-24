import { ReadableStream } from 'web-streams-polyfill/ponyfill'

import { ReadableStreamBuffer } from 'stream-buffers'

export function arrayToStream (array) {
    const myReadableStreamBuffer = new ReadableStreamBuffer({
        frequency: 10,   // in milliseconds.
        chunkSize: 2048  // in bytes.
    })

    myReadableStreamBuffer.put(Buffer.from(array))
    myReadableStreamBuffer.stop()

    const webReader = nodeToWeb(myReadableStreamBuffer)

    return webReader
}

function nodeToWeb (nodeStream) {
    var destroyed = false
    var listeners = {}
  
    function start (controller) {
      listeners['data'] = onData
      listeners['end'] = onData
      listeners['end'] = onDestroy
      listeners['close'] = onDestroy
      listeners['error'] = onDestroy
      for (var name in listeners) nodeStream.on(name, listeners[name])
  
      nodeStream.pause()
  
      function onData (chunk) {
        if (destroyed) return
        controller.enqueue(chunk)
        nodeStream.pause()
      }
  
      function onDestroy (err) {
        if (destroyed) return
        destroyed = true
  
        for (var name in listeners) nodeStream.removeListener(name, listeners[name])
  
        if (err) controller.error(err)
        else controller.close()
      }
    }
  
    function pull () {
      if (destroyed) return
      nodeStream.resume()
    }
  
    function cancel () {
      destroyed = true
  
      for (var name in listeners) nodeStream.removeListener(name, listeners[name])
  
      nodeStream.push(null)
      nodeStream.pause()
      if (nodeStream.destroy) nodeStream.destroy()
      else if (nodeStream.close) nodeStream.close()
    }
  
    return new ReadableStream({start: start, pull: pull, cancel: cancel})
  }
  