package flatgeobuf

// Behavior defines the way the flatgeobuf file will be handled.
type Behavior int

const (
	// BehaviorUnknown is the default. It has no effect at all when used.
	BehaviorUnknown Behavior = 0

	// BehaviorMMapAll forces all data to be used directly from the mmapped
	// file. Incompatible with BehaviorLoadAll being set.
	BehaviorMMapAll Behavior = 1 << (iota - 1)

	// BehaviorLoadAll forces all data to be loaded into memory. Incompatible
	// with BehaviorMMapAll being set.
	BehaviorLoadAll

	// BehaviorLoadIndex copies the index from the mmaped file to RAM so
	// that it will always be available. Note that this will increase overall
	// memory usage (cached data + in-memory data). Does nothing if
	// BehaviorLoadAll is set.
	BehaviorLoadIndex

	// BehaviorPrefault forces mmapped data to be prefaulted. This will load
	// as much data as possible from the mmaped file into de disk cache memory.
	// Does nothing if BehaviorMMapAll is not set.
	BehaviorPrefault
)
