package flatgeobuf

import (
	"bytes"
	"encoding/binary"
	"reflect"
	"testing"

	"github.com/flatgeobuf/flatgeobuf/src/go/flattypes"
	"github.com/flatgeobuf/flatgeobuf/src/go/writer"
	flatbuffers "github.com/google/flatbuffers/go"
)

func TestFlatGeoBuf(t *testing.T) {
	fgb, err := New("./poly_landmarks.fgb")
	if err != nil {
		t.Errorf("expected nil error, got: %v", err)
	}

	features, _ := fgb.Search(-73.976523, 40.715091, -73.971893, 40.727318)

	if len(features) != 2 {
		t.Errorf("expected 2 features, got: %v", len(features))
	}
}

func TestFlatGeoBuf_Prefault(t *testing.T) {
	fgb, err := NewWithBehavior("./poly_landmarks.fgb", BehaviorMMapAll|BehaviorPrefault)
	if err != nil {
		t.Errorf("expected nil error, got: %v", err)
	}

	features, _ := fgb.Search(-73.976523, 40.715091, -73.971893, 40.727318)

	if len(features) != 2 {
		t.Errorf("expected 2 features, got: %v", len(features))
	}
}

func TestFlatGeoBuf_LoadAll(t *testing.T) {
	fgb, err := NewWithBehavior("./poly_landmarks.fgb", BehaviorLoadAll)
	if err != nil {
		t.Errorf("expected nil error, got: %v", err)
	}

	features, _ := fgb.Search(-73.976523, 40.715091, -73.971893, 40.727318)

	if len(features) != 2 {
		t.Errorf("expected 2 features, got: %v", len(features))
	}
}

func TestFlatGeoBuf_LoadIndex(t *testing.T) {
	fgb, err := NewWithBehavior("./poly_landmarks.fgb", BehaviorMMapAll|BehaviorLoadIndex)
	if err != nil {
		t.Errorf("expected nil error, got: %v", err)
	}

	features, _ := fgb.Search(-73.976523, 40.715091, -73.971893, 40.727318)

	if len(features) != 2 {
		t.Errorf("expected 2 features, got: %v", len(features))
	}
}

func TestCreateFGBFileAndBasicSearch(t *testing.T) {
	// SETUP:
	// Create a mock fgb file
	//

	// four features that are the standard quadrants of the Cartesian plane, unit-sized
	// the uint32 property attached to each is the quadrant number
	fgen := func() *fg {
		return &fg{
			Features: []*writer.Feature{
				createSquareFeature(0, 0, 1, 1, 1),
				createSquareFeature(-1, 0, 0, 1, 2),
				createSquareFeature(-1, -1, 0, 0, 3),
				createSquareFeature(0, -1, 1, 0, 4),
			},
		}
	}

	hu := &hu{
		MetadataStr: `{"TotalHouseholds": 10}`,
	}

	hgen := func() *writer.Header {
		headerBuilder := flatbuffers.NewBuilder(0)
		header := writer.NewHeader(headerBuilder).
			SetName("Households ShapeFile Data").
			SetTitle("Households ShapeFile Data").
			SetGeometryType(flattypes.GeometryTypePolygon)
		householdsCol := writer.NewColumn(headerBuilder).
			SetName("Households").
			SetType(flattypes.ColumnTypeUInt)
		header.SetColumns([]*writer.Column{householdsCol})
		return header
	}

	header := hgen()

	var mockFile bytes.Buffer
	wr := writer.NewWriter(header, true, fgen(), hu)
	wr.Write(&mockFile)

	// TEST:
	// check header metadata
	// run search cases where we expect 0, 1, 2, and 4 results
	//

	fgbFile, err := NewWithData(mockFile.Bytes())
	if err != nil {
		t.Fatalf("failed to create FlatGeoBuf: %v", err)
	}

	meta := string(fgbFile.header.Metadata())
	if meta != hu.MetadataStr {
		t.Errorf("Incorrect header metadata: got %q, want %q", meta, hu.MetadataStr)
	}

	type test struct {
		searchMinX float64
		searchMinY float64
		searchMaxX float64
		searchMaxY float64

		expectedResultProperties []int
	}

	tests := []test{
		{searchMinX: 0.5, searchMinY: 0.5, searchMaxX: 0.6, searchMaxY: 0.6,
			expectedResultProperties: []int{1}},
		{searchMinX: -0.6, searchMinY: -0.1, searchMaxX: -0.5, searchMaxY: 0.1,
			expectedResultProperties: []int{2, 3}},
		{searchMinX: -0.1, searchMinY: -0.1, searchMaxX: 0.1, searchMaxY: 0.1,
			expectedResultProperties: []int{1, 2, 3, 4}},
		{searchMinX: 2, searchMinY: 2, searchMaxX: 3, searchMaxY: 3,
			expectedResultProperties: []int{}},
	}

	for _, test := range tests {
		features, _ := fgbFile.Search(test.searchMinX, test.searchMinY, test.searchMaxX, test.searchMaxY)
		if got, want := len(features), len(test.expectedResultProperties); got != want {
			t.Errorf("Search(%0.1f, %0.1f, %0.1f, %0.1f) returned %d features. Expected %d",
				test.searchMinX, test.searchMinY, test.searchMaxX, test.searchMaxY, got, want)
		}
		gotSet, wantSet := make(map[int]bool), make(map[int]bool)
		for _, want := range test.expectedResultProperties {
			wantSet[want] = true
		}
		for _, f := range features {
			prop := binary.LittleEndian.Uint16(f.PropertiesBytes()[1:])
			gotSet[int(prop)] = true
		}
		if !reflect.DeepEqual(gotSet, wantSet) {
			t.Errorf("Unexpected search results. (want = %v, got = %v)", wantSet, gotSet)
		}
	}

	// Check writer.WithMemory produces the same result
	var got bytes.Buffer
	mwr := writer.NewWriter(hgen(), true, fgen(), hu, writer.WithMemory())
	mwr.Write(&got)
	if !reflect.DeepEqual(got.Bytes(), mockFile.Bytes()) {
		t.Error("Unexpected results using writer.WithMemory")
	}
}

func createSquareFeature(xmin, ymin, xmax, ymax float64, propertyCount uint16) *writer.Feature {
	properties := make([]byte, 5)
	binary.LittleEndian.PutUint16(properties[1:], propertyCount)

	featureBuilder := flatbuffers.NewBuilder(0)
	geo := writer.NewGeometry(featureBuilder).SetXY([]float64{
		xmin, ymin,
		xmin, ymax,
		xmax, ymax,
		xmax, ymin,
	})
	return writer.NewFeature(featureBuilder).SetProperties(properties).SetGeometry(geo)
}

type fg struct {
	Features []*writer.Feature
	idx      int
}

var _ writer.FeatureGenerator = (*fg)(nil)

func (fg *fg) Generate() *writer.Feature {
	var feature *writer.Feature
	if fg.idx < len(fg.Features) {
		feature = fg.Features[fg.idx]
		fg.idx++
	}
	return feature
}

type hu struct {
	MetadataStr string
}

var _ writer.HeaderUpdater = (*hu)(nil)

func (hu *hu) Update(header *writer.Header) {
	header.SetMetadata(hu.MetadataStr)
}
