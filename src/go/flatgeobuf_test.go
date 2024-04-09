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
	if _, err := wr.Write(&mockFile); err != nil {
		t.Errorf("failed to write in buffer: %v", err)
	}

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

	tests := []testSearch{
		{searchMinX: 0.5, searchMinY: 0.5, searchMaxX: 0.6, searchMaxY: 0.6,
			expectedResultProperties: []int{1}},
		{searchMinX: -0.6, searchMinY: -0.1, searchMaxX: -0.5, searchMaxY: 0.1,
			expectedResultProperties: []int{2, 3}},
		{searchMinX: -0.1, searchMinY: -0.1, searchMaxX: 0.1, searchMaxY: 0.1,
			expectedResultProperties: []int{1, 2, 3, 4}},
		{searchMinX: 2, searchMinY: 2, searchMaxX: 3, searchMaxY: 3,
			expectedResultProperties: []int{}},
	}

	runTestSearchInFGB(t, fgbFile, tests)

	// Check writer.WithMemory produces the same result
	var got bytes.Buffer
	mwr := writer.NewWriter(hgen(), true, fgen(), hu, writer.WithMemory())
	if _, err = mwr.Write(&got); err != nil {
		t.Errorf("failed to write with WithMemory: %v", err)
	}

	if !reflect.DeepEqual(got.Bytes(), mockFile.Bytes()) {
		t.Error("Unexpected results using writer.WithMemory")
	}
}

func TestFGBFileWithMultiPolygon(t *testing.T) {
	// SETUP:
	// Create a mock fgb file
	//

	// four multipolygon features that are composed of squares, each
	// feature being in a quadrant of the cartesian plan, the uint32 property
	// attached to each is the quadrant number
	fgen := func() *fg {
		return &fg{
			Features: []*writer.Feature{
				createMultiSquareFeature([][4]float64{{1, 1, 2, 2}, {2, 2, 3, 3}}, 1),
				createMultiSquareFeature([][4]float64{{-2, 1, -1, 2}, {-3, 2, -2, 3}}, 2),
				createMultiSquareFeature([][4]float64{{-2, -2, -1, -1}, {-2, -2, -3, -3}}, 3),
				createMultiSquareFeature([][4]float64{{1, -2, 2, -1}, {2, -3, 3, -2}}, 4),
			},
		}
	}

	hgen := func() *writer.Header {
		headerBuilder := flatbuffers.NewBuilder(0)
		header := writer.NewHeader(headerBuilder).
			SetName("Households ShapeFile Data").
			SetTitle("Households ShapeFile Data").
			SetGeometryType(flattypes.GeometryTypeMultiPolygon)
		householdsCol := writer.NewColumn(headerBuilder).
			SetName("Households").
			SetType(flattypes.ColumnTypeUInt)
		header.SetColumns([]*writer.Column{householdsCol})
		return header
	}

	header := hgen()

	var mockFile bytes.Buffer
	wr := writer.NewWriter(header, true, fgen(), nil)
	if _, err := wr.Write(&mockFile); err != nil {
		t.Errorf("failed to write in buffer: %v", err)
	}

	// TEST:
	// run search cases where we expect 0, 1, 2, and 4 results
	//

	fgbFile, err := NewWithData(mockFile.Bytes())
	if err != nil {
		t.Fatalf("failed to create FlatGeoBuf: %v", err)
	}

	tests := []testSearch{
		{searchMinX: 1.5, searchMinY: 1.5, searchMaxX: 1.6, searchMaxY: 1.6,
			expectedResultProperties: []int{1}},
		{searchMinX: -1.6, searchMinY: -1.1, searchMaxX: -1.5, searchMaxY: 1.1,
			expectedResultProperties: []int{2, 3}},
		{searchMinX: -1.1, searchMinY: -1.1, searchMaxX: 1.1, searchMaxY: 1.1,
			expectedResultProperties: []int{1, 2, 3, 4}},
		{searchMinX: 3.5, searchMinY: 3.5, searchMaxX: 4.5, searchMaxY: 4.5,
			expectedResultProperties: []int{}},
	}

	runTestSearchInFGB(t, fgbFile, tests)
}

func runTestSearchInFGB(t *testing.T, fgbFile *FlatGeoBuf, tests []testSearch) {
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
}

type testSearch struct {
	searchMinX float64
	searchMinY float64
	searchMaxX float64
	searchMaxY float64

	expectedResultProperties []int
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

func createMultiSquareFeature(multiSquares [][4]float64, propertyCount uint16) *writer.Feature {
	properties := make([]byte, 5)
	binary.LittleEndian.PutUint16(properties[1:], propertyCount)

	featureBuilder := flatbuffers.NewBuilder(0)
	parts := make([]writer.Geometry, len(multiSquares))
	for i, coords := range multiSquares {
		xmin, ymin, xmax, ymax := coords[0], coords[1], coords[2], coords[3]
		parts[i] = *writer.NewGeometry(featureBuilder).SetXY([]float64{
			xmin, ymin,
			xmin, ymax,
			xmax, ymax,
			xmax, ymin,
		})
	}
	geo := writer.NewGeometry(featureBuilder).SetXY([]float64{}).SetParts(parts)
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
