package writer

// FeatureGenerator is an interface for generating features.
type FeatureGenerator interface {
	// Generate generates a feature. Returns nil if there are no more features
	// to generate.
	Generate() *Feature
}
