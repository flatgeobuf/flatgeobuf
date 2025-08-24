using System.Buffers;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace NetTopologySuite.Geometries.Implementation
{
    internal sealed class CastingMemoryManager<T>(ArraySegment<byte> data) : MemoryManager<T>
            where T : unmanaged
    {
        private ArraySegment<byte> _data = data;

        private bool _disposed;

        public override Span<T> GetSpan() => MemoryMarshal.Cast<byte, T>(_data);

        public override unsafe MemoryHandle Pin(int elementIndex = 0)
        {
            ObjectDisposedException.ThrowIf(_disposed, nameof(CastingMemoryManager<T>));
            if ((uint)elementIndex > (uint)_data.Count)
                throw new ArgumentOutOfRangeException(nameof(elementIndex));
            var handle = GCHandle.Alloc(_data.Array, GCHandleType.Pinned);
            return new MemoryHandle(Unsafe.Add<T>((void*)handle.AddrOfPinnedObject(), _data.Offset + elementIndex), handle, this);
        }

        public override void Unpin()
        {
        }

        protected override void Dispose(bool disposing)
        {
            if (_disposed)
                return;
            if (disposing)
                _data = default;
            _disposed = true;
        }
    }
}