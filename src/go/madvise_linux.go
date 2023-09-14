package flatgeobuf

import (
	"syscall"
)

func madvise(b []byte, advice int) error {
	// Linux requires page alignmment otherwise the Madvise call will
	// fail.
	return syscall.Madvise(b, advice)
}
