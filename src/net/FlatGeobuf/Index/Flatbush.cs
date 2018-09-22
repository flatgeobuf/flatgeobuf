using System;

using FlatBuffers;
using FlatGeobuf.NTS;

namespace FlatGeobuf
{
    public class Flatbush
    {
        double minX;
        double minY;
        double maxX;
        double maxY;

        double[] _boxes;
        uint[] _indices;
        uint _pos;

        uint[] _levelBounds;


        static void sort(uint[] values, double[] boxes, uint[] indices, uint left, uint right) {
            if (left >= right) return;

            var pivot = values[(left + right) >> 1];
            var i = left - 1;
            var j = right + 1;

            while (true) {
                do i++; while (values[i] < pivot);
                do j--; while (values[j] > pivot);
                if (i >= j) break;
                swap(values, boxes, indices, i, j);
            }

            sort(values, boxes, indices, left, j);
            sort(values, boxes, indices, j + 1, right);
        }

        static void swap(uint[] values, double[] boxes, uint[] indices, uint i, uint j) {
            var temp = values[i];
            values[i] = values[j];
            values[j] = temp;

            var k = 4 * i;
            var m = 4 * j;

            var a = boxes[k];
            var b = boxes[k + 1];
            var c = boxes[k + 2];
            var d = boxes[k + 3];
            boxes[k] = boxes[m];
            boxes[k + 1] = boxes[m + 1];
            boxes[k + 2] = boxes[m + 2];
            boxes[k + 3] = boxes[m + 3];
            boxes[m] = a;
            boxes[m + 1] = b;
            boxes[m + 2] = c;
            boxes[m + 3] = d;

            var e = indices[i];
            indices[i] = indices[j];
            indices[j] = e;
        }

        static uint hilbert(int x, int y) {
            var a = x ^ y;
            var b = 0xFFFF ^ a;
            var c = 0xFFFF ^ (x | y);
            var d = x & (y ^ 0xFFFF);

            var A = a | (b >> 1);
            var B = (a >> 1) ^ a;
            var C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
            var D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

            a = A; b = B; c = C; d = D;
            A = ((a & (a >> 2)) ^ (b & (b >> 2)));
            B = ((a & (b >> 2)) ^ (b & ((a ^ b) >> 2)));
            C ^= ((a & (c >> 2)) ^ (b & (d >> 2)));
            D ^= ((b & (c >> 2)) ^ ((a ^ b) & (d >> 2)));

            a = A; b = B; c = C; d = D;
            A = ((a & (a >> 4)) ^ (b & (b >> 4)));
            B = ((a & (b >> 4)) ^ (b & ((a ^ b) >> 4)));
            C ^= ((a & (c >> 4)) ^ (b & (d >> 4)));
            D ^= ((b & (c >> 4)) ^ ((a ^ b) & (d >> 4)));

            a = A; b = B; c = C; d = D;
            C ^= ((a & (c >> 8)) ^ (b & (d >> 8)));
            D ^= ((b & (c >> 8)) ^ ((a ^ b) & (d >> 8)));

            a = C ^ (C >> 1);
            b = D ^ (D >> 1);

            var i0 = x ^ y;
            var i1 = b | (0xFFFF ^ (i0 | a));

            i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
            i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
            i0 = (i0 | (i0 << 2)) & 0x33333333;
            i0 = (i0 | (i0 << 1)) & 0x55555555;

            i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
            i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
            i1 = (i1 | (i1 << 2)) & 0x33333333;
            i1 = (i1 | (i1 << 1)) & 0x55555555;

            return (uint) ((i1 << 1) | i0) >> 0;
        }
    }
}
