package flatgeobuf

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"runtime"
	"syscall"

	"github.com/flatgeobuf/flatgeobuf/src/go/flattypes"
	"github.com/flatgeobuf/flatgeobuf/src/go/index"
	"github.com/flatgeobuf/flatgeobuf/src/go/writer"

	flatbuffers "github.com/google/flatbuffers/go"
)

// FlatGeoBuf allows read-only handling of a flatgeobuf file.
type FlatGeoBuf struct {
	data    []byte
	mmapped bool

	header *flattypes.Header
	index  *index.PackedRTree

	featuresOffset int
}

// New creates a new FlatGeoBuf instance from a file path by memory mapping the
// file.
func New(path string) (*FlatGeoBuf, error) {
	return NewWithBehavior(path, BehaviorMMapAll)
}

// NewWithBehavior creates a new FlatGeoBuf instance from a file path with the
// given behavior.
func NewWithBehavior(path string, behavior Behavior) (*FlatGeoBuf, error) {
	loadAll := behavior&BehaviorLoadAll != 0
	mmapAll := behavior&BehaviorMMapAll != 0
	if loadAll && mmapAll {
		return nil, fmt.Errorf("behaviors BehaviorLoadAll and BehaviorMMapAll " +
			"are incompatible")
	}

	if !loadAll && !mmapAll {
		return nil, fmt.Errorf("either BehaviorLoadAll or BehaviorMMapAll must " +
			"be set")
	}

	fgb := &FlatGeoBuf{
		mmapped: mmapAll,
	}

	err := fgb.mmapOrLoadFile(path, behavior)
	if err != nil {
		return nil, fmt.Errorf("error obtaining data from file: %w", err)
	}

	err = fgb.setup(behavior)
	if err != nil {
		return nil, fmt.Errorf("error setting up flatgeobuf: %w", err)
	}

	return fgb, nil
}

// NewWithMockedData creates a new FlatGeoBuf from the given byte slice. The
// contents of the slice should be the same of a valid flatgeobuf file.
func NewWithData(data []byte) (*FlatGeoBuf, error) {
	fgb := &FlatGeoBuf{
		data:    data,
		mmapped: false,
	}

	err := fgb.setup(BehaviorLoadAll)
	if err != nil {
		return nil, err
	}
	return fgb, nil
}

// Header allows access to the underlying flatgeobuf header in flatbuffer format.
func (fgb *FlatGeoBuf) Header() *flattypes.Header {
	return fgb.header
}

// Search allows searching for data with the index built into the flatgeobuf
// file. If the file does not include an index, returns a nil Feature slice and
// a non-nil error. Otherwise returns a slice of the matched Features and a nil
// error.
func (fgb *FlatGeoBuf) Search(minX, minY, maxX, maxY float64) ([]*flattypes.Feature, error) {
	if fgb.index == nil {
		return nil, fmt.Errorf("no index present in flatgeobuf file")
	}

	hits := fgb.index.Search(minX, minY, maxX, maxY)

	features := make([]*flattypes.Feature, len(hits))
	for i, hit := range hits {
		startFeatureOffset := uint64(fgb.featuresOffset) + hit.Offset

		features[i] = flattypes.GetSizePrefixedRootAsFeature(fgb.data,
			flatbuffers.UOffsetT(startFeatureOffset))
	}

	return features, nil
}

func (fgb *FlatGeoBuf) close() {
	if fgb.data == nil {
		return
	}

	if fgb.mmapped {
		// Data was mmaped. Some cleanup is needed.
		data := fgb.data
		fgb.data = nil

		runtime.SetFinalizer(fgb, nil)
		syscall.Munmap(data)
	}
}

func (fgb *FlatGeoBuf) mmapOrLoadFile(path string, behavior Behavior) error {
	fileInfo, err := os.Stat(path)
	if err != nil {
		return err
	}

	if fileInfo.IsDir() {
		return fmt.Errorf("path is a directory")
	}

	size := fileInfo.Size()

	if size == 0 {
		return fmt.Errorf("file is empty")
	}
	if size < 0 {
		return fmt.Errorf("file %q has negative size", path)
	}
	if int64(size) != int64(int(size)) {
		return fmt.Errorf("file %q is too large", path)
	}

	f, err := os.Open(path)
	if err != nil {
		return err
	}
	defer f.Close()

	if behavior&BehaviorMMapAll != 0 {
		// File should be mmaped.
		fgb.data, err = syscall.Mmap(int(f.Fd()), 0, int(size),
			syscall.PROT_READ, syscall.MAP_PRIVATE)
		if err != nil {
			return fmt.Errorf("error mmapping file: %w", err)
		}
		runtime.SetFinalizer(fgb, (*FlatGeoBuf).close)

		madvFlags := syscall.MADV_RANDOM
		if behavior&BehaviorPrefault != 0 {
			// And we want to prefault it.
			madvFlags |= syscall.MADV_WILLNEED
		}

		madvise(fgb.data, madvFlags)
	} else {
		// File should be loaded.
		fgb.data, err = io.ReadAll(f)
		if err != nil {
			return fmt.Errorf("error loading file: %w", err)
		}
	}
	return nil
}

func (fgb *FlatGeoBuf) setup(behavior Behavior) error {
	// Check magic bytes.
	if len(fgb.data) < len(writer.MagicBytes) ||
		!bytes.Equal(fgb.data[:len(writer.MagicBytes)], writer.MagicBytes) {
		return fmt.Errorf("not a flatgeobuf file: invalid magic bytes")
	}

	// Increment offset past magic bytes.
	offset := len(writer.MagicBytes)

	// Read header size.
	headerSize := int(flatbuffers.GetUOffsetT(fgb.data[offset:]))

	// Read header ignoring the size
	fgb.header = flattypes.GetSizePrefixedRootAsHeader(fgb.data, flatbuffers.UOffsetT(offset))

	// Increment offset past header.
	offset += headerSize + flatbuffers.SizeUOffsetT

	indexNodeSize := fgb.header.IndexNodeSize()
	if indexNodeSize > 0 {
		// We have an index.

		// Figure out if we should copy the iondex to memory or just mmap it.
		copyData := behavior&BehaviorMMapAll != 0 && behavior&BehaviorLoadIndex != 0

		// Create our Index instance.
		fgb.index = index.NewPackedRTreeFromData(fgb.data[offset:],
			uint64(fgb.header.FeaturesCount()), indexNodeSize, copyData)

		// Increment offset past index.
		offset += int(fgb.index.Size())
	}

	fgb.featuresOffset = offset

	return nil
}
