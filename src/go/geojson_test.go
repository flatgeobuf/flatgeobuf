package geojson

import (
	"testing"
)

func TestPoint(t *testing.T) {
	p := Point{1, 2}
	if v := p.Lon(); v != 1 {
		t.Errorf("incorrect lon: %v != 1", v)
	}

	if v := p.Lat(); v != 2 {
		t.Errorf("incorrect lat: %v != 2", v)
	}
}
