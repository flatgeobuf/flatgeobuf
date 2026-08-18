package flatgeobuf

import (
	"encoding/binary"
	"testing"

	"github.com/flatgeobuf/flatgeobuf/src/go/writer"
)

// TestNewWithData_TruncatedHeader ensures that truncated input carrying only a
// valid magic-byte prefix is rejected with an error instead of panicking with
// an out-of-bounds slice access while reading the header size prefix.
func TestNewWithData_TruncatedHeader(t *testing.T) {
	cases := map[string][]byte{
		"magic only":              writer.MagicBytes,
		"magic plus partial size": append(append([]byte{}, writer.MagicBytes...), 0x00, 0x00),
	}

	for name, data := range cases {
		t.Run(name, func(t *testing.T) {
			if _, err := NewWithData(data); err == nil {
				t.Fatalf("expected error for truncated input, got nil")
			}
		})
	}
}

// TestNewWithData_HeaderSizeExceedsData ensures that a header size prefix
// claiming more bytes than the buffer contains is rejected with an error
// instead of letting flatbuffers read out of bounds.
func TestNewWithData_HeaderSizeExceedsData(t *testing.T) {
	data := append([]byte{}, writer.MagicBytes...)
	size := make([]byte, 4)
	binary.LittleEndian.PutUint32(size, 1<<20) // claim a 1 MiB header
	data = append(data, size...)

	if _, err := NewWithData(data); err == nil {
		t.Fatalf("expected error for oversized header size, got nil")
	}
}
