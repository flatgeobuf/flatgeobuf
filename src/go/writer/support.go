package writer

import flatbuffers "github.com/google/flatbuffers/go"

func maybeCreateString(b *flatbuffers.Builder, s string) flatbuffers.UOffsetT {
	if s != "" {
		return b.CreateString(s)
	}

	return 0
}
