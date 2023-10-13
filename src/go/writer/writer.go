package writer

import (
	"bytes"
	"io"
	"os"

	"github.com/flatgeobuf/flatgeobuf/src/go/index"
)

// MagicBytes is the identifier sequence for a flatgeobuf file.
var MagicBytes = []byte{0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00}

// Writer is a type that allows constructing a valid flatgeobuf file that has a
// Header, an optional Index and collection of Features.
type Writer struct {
	header *Header

	featureGenerator FeatureGenerator

	headerUpdater HeaderUpdater

	includeIndex bool

	useMemory bool
}

// featureItem is an Index entry for a specific Feature.
type featureItem struct {
	nodeItem *index.NodeItem

	size   int
	offset uint64
}

func (f *featureItem) NodeItem() index.NodeItem {
	return *f.nodeItem
}

// writerOption is the additional option to be passed into Writer.
type writerOption func(*Writer)

// Use memory instead of temporary file when generating index
// or execute HeaderUpdater.
// Effective only when includeIndex is true or when HeaderUpdater is provided.
//
// Warning: this option could use arbitrarily large amounts of memory.
func WithMemory() writerOption {
	return func(w *Writer) {
		w.useMemory = true
	}
}

// NewWriter returns a new writer instance that will write a flatgeobuf file
// with the given Header, a possible Index (depending on includeIndex), a
// FeatureGenerator that will provide the Features to be written and a
// HeaderUpdater that will be used to update the Header after all features
// have been generated.
func NewWriter(header *Header, includeIndex bool,
	featureGenerator FeatureGenerator, headerUpdater HeaderUpdater,
	opts ...writerOption,
) *Writer {
	w := &Writer{
		header:           header,
		featureGenerator: featureGenerator,
		headerUpdater:    headerUpdater,
		includeIndex:     includeIndex,
	}

	for _, opt := range opts {
		opt(w)
	}

	return w
}

func writeFeature(feature *Feature, w io.Writer) (int, error) {
	featureOffset := feature.Build()
	feature.builder.FinishSizePrefixed(featureOffset)
	return w.Write(feature.builder.FinishedBytes())
}

// Write writes the flatgeobuf file represented by the given io.Writer.
func (w *Writer) Write(ioWriter io.Writer) (int, error) {
	totalBytesWritten := 0

	// Write magic bytes to destination file.
	n, err := ioWriter.Write(MagicBytes)
	totalBytesWritten += n
	if err != nil {
		return totalBytesWritten, err
	}

	if !w.includeIndex && w.headerUpdater == nil {
		// We are not including the index nor are we updating the header after
		// adding entries so we just write the Header as-is.
		headerOffset := w.header.Build()
		w.header.builder.FinishSizePrefixed(headerOffset)
		n, err = ioWriter.Write(w.header.builder.FinishedBytes())
		totalBytesWritten += n
		if err != nil {
			return totalBytesWritten, err
		}

		// And now we write all the features returned by the given generator.
		for feature := w.featureGenerator.Generate(); feature != nil; feature =
			w.featureGenerator.Generate() {
			n, err = writeFeature(feature, ioWriter)
			totalBytesWritten += n
			if err != nil {
				return totalBytesWritten, err
			}
		}
	} else {
		// We have an index, a header updater or both. We will need to add it
		// and also adjust the header to match.

		// Create a temporary io.Writer to keep the generated features.
		var featureStore featureStorer
		var err error

		if w.useMemory {
			featureStore, err = newMemoryBuffer()
		} else {
			featureStore, err = newTempFile("flatgeobuf_features_")
		}
		if err != nil {
			return 0, err
		}

		defer featureStore.Close()

		// Add the features to the temporary file and collect tems for the index.
		featureOffset := uint64(0)
		items := []index.Item{}
		extent := index.NewNodeItem(0)
		for feature := w.featureGenerator.Generate(); feature != nil; feature = w.featureGenerator.Generate() {
			n, err = writeFeature(feature, featureStore)
			if err != nil {
				return totalBytesWritten, err
			}

			exGeometry := NewGeometryExtended(feature.geometry)
			minX, minY, maxX, maxY := exGeometry.BoundingBox()

			nodeItem := index.NewNodeItemWithCoordinates(featureOffset, minX, minY, maxX, maxY)
			item := &featureItem{
				&nodeItem,
				n,
				featureOffset,
			}

			items = append(items, item)

			extent.Expand(nodeItem)

			featureOffset += uint64(n)
		}
		err = featureStore.Sync()
		if err != nil {
			return totalBytesWritten, err
		}

		// Adjust and write header.
		envelope := extent.ToSlice()
		w.header.SetEnvelope(envelope)
		w.header.SetIndexNodeSize(16)
		w.header.SetFeaturesCount(uint64(len(items)))

		// Call our header updater if we have one.
		if w.headerUpdater != nil {
			w.headerUpdater.Update(w.header)
		}

		headerOffset := w.header.Build()
		w.header.builder.FinishSizePrefixed(headerOffset)
		n, err = ioWriter.Write(w.header.builder.FinishedBytes())
		totalBytesWritten += n
		if err != nil {
			return totalBytesWritten, err
		}

		// Create and write index.
		index.HilbertSortItems(items)

		featureOffset = 0
		for _, item := range items {
			item.(*featureItem).offset = featureOffset
			featureOffset += uint64(item.(*featureItem).size)
		}

		tree := index.NewPackedRTreeWithItems(items, extent, 16)
		n, err := tree.Write(ioWriter)
		totalBytesWritten += n
		if err != nil {
			return totalBytesWritten, err
		}

		_, err = featureStore.Seek(0, 0)
		if err != nil {
			return totalBytesWritten, err
		}

		// Copy features from temporary file to destination file.
		var written int64
		if written, err = io.Copy(ioWriter, featureStore); err != nil {
			return totalBytesWritten, err
		}

		totalBytesWritten += int(written)
	}

	return totalBytesWritten, nil
}

type featureStorer interface {
	io.WriteCloser
	io.ReadSeeker
	Sync() error
}

type tempFile struct {
	*os.File
}

func newTempFile(pattern string) (*tempFile, error) {
	f, err := os.CreateTemp("", pattern)
	if err != nil {
		return nil, err
	}

	return &tempFile{
		File: f,
	}, nil
}

func (f *tempFile) Close() error {
	if err := f.File.Close(); err != nil {
		return err
	}

	return os.Remove(f.Name())
}

type memoryBuffer struct {
	*bytes.Buffer
}

func newMemoryBuffer() (*memoryBuffer, error) {
	return &memoryBuffer{
		Buffer: &bytes.Buffer{},
	}, nil
}

func (b *memoryBuffer) Seek(_offset int64, _whence int) (int64, error) {
	return 0, nil
}

func (b *memoryBuffer) Close() error {
	return nil
}

func (b *memoryBuffer) Sync() error {
	return nil
}
